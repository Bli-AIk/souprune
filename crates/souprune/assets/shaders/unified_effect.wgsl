// Unified effect shader - combines multiple effects in a single pass
//
// This shader supports five effects that can be enabled/disabled via flags:
// 1. Mask clipping (rectangular region)
// 2. Wipe transition (progressive reveal/hide)
// 3. Stretch segment (UV domain distortion)
// 4. Gaussian blur (optimized cross-shaped sampling)
// 5. Palette map (color quantization to palette)
//
// Each effect can be toggled on/off via the effect_flags uniform.
//
// Uniform layout:
// 0: color (vec4) - tint color
// 1: effect_flags (vec4) - (mask_enabled, wipe_enabled, stretch_enabled, blur_enabled)
// 2: mask_params (vec4) - (center_x, center_y, half_width, half_height)
// 3: wipe_params (vec4) - (wipe_start, wipe_end, wipe_angle, wipe_feather)
// 4: stretch_params (vec4) - (angle_radians, stretch_px, offset_px, smooth_width)
// 5: original_size (vec4) - (orig_width, orig_height, mesh_width, mesh_height)
// 6: mesh_offset (vec4) - (center_offset_x, center_offset_y, 0, 0)
// 9: blur_params (vec4) - (radius_px, orig_width, orig_height, expansion_px)
// 10: palette_flags (vec4) - (enabled, count, shades, alpha)
// 11-18: palette_colors 1-8

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> effect_flags: vec4<f32>;
@group(2) @binding(2) var<uniform> mask_params: vec4<f32>;
@group(2) @binding(3) var<uniform> wipe_params: vec4<f32>;
@group(2) @binding(4) var<uniform> stretch_params: vec4<f32>;
@group(2) @binding(5) var<uniform> original_size: vec4<f32>;
@group(2) @binding(6) var<uniform> mesh_offset: vec4<f32>;
@group(2) @binding(7) var base_texture: texture_2d<f32>;
@group(2) @binding(8) var base_sampler: sampler;
@group(2) @binding(9) var<uniform> blur_params: vec4<f32>;
@group(2) @binding(10) var<uniform> palette_flags: vec4<f32>;
@group(2) @binding(11) var<uniform> palette_color1: vec4<f32>;
@group(2) @binding(12) var<uniform> palette_color2: vec4<f32>;
@group(2) @binding(13) var<uniform> palette_color3: vec4<f32>;
@group(2) @binding(14) var<uniform> palette_color4: vec4<f32>;
@group(2) @binding(15) var<uniform> palette_color5: vec4<f32>;
@group(2) @binding(16) var<uniform> palette_color6: vec4<f32>;
@group(2) @binding(17) var<uniform> palette_color7: vec4<f32>;
@group(2) @binding(18) var<uniform> palette_color8: vec4<f32>;
@group(2) @binding(19) var<uniform> mask2_params: vec4<f32>;
@group(2) @binding(20) var<uniform> mask2_flags: vec4<f32>;

// Helper: rotate 2D vector by angle
fn rotate_vec(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(
        v.x * c - v.y * s,
        v.x * s + v.y * c
    );
}

// Apply stretch segment effect - returns modified UV
fn apply_stretch_segment(uv: vec2<f32>) -> vec2<f32> {
    let angle = stretch_params.x;
    let stretch_px = stretch_params.y;
    let offset_px = stretch_params.z;
    
    let orig_width = original_size.x;
    let orig_height = original_size.y;
    let mesh_width = original_size.z;
    let mesh_height = original_size.w;
    
    let center_off_x = mesh_offset.x;
    let center_off_y = mesh_offset.y;
    
    let angle_factor = mix(1.0, 0.9, abs(sin(angle)));
    let half_gap = stretch_px * 0.5 * angle_factor;
    
    let pixel_x = (uv.x - 0.5) * mesh_width + center_off_x;
    let pixel_y = (uv.y - 0.5) * mesh_height + center_off_y;
    let pixel_coord = vec2<f32>(pixel_x, pixel_y);
    
    let rotated_pixel = rotate_vec(pixel_coord, angle);
    let shifted_x = rotated_pixel.x + offset_px;
    
    var sample_rotated_x: f32;
    if abs(shifted_x) < half_gap {
        sample_rotated_x = -offset_px;
    } else {
        sample_rotated_x = rotated_pixel.x - sign(shifted_x) * half_gap;
    }
    
    let final_rotated = vec2<f32>(sample_rotated_x, rotated_pixel.y);
    let unrotated_pixel = rotate_vec(final_rotated, -angle);
    
    return vec2<f32>(
        (unrotated_pixel.x / orig_width) + 0.5,
        (unrotated_pixel.y / orig_height) + 0.5
    );
}

// Apply wipe effect - returns alpha multiplier
fn apply_wipe(uv: vec2<f32>) -> f32 {
    let wipe_start = wipe_params.x;
    let wipe_end = wipe_params.y;
    let wipe_angle = wipe_params.z;
    let wipe_feather = wipe_params.w;
    
    let cos_angle = cos(wipe_angle);
    let sin_angle = sin(wipe_angle);
    let centered_uv = uv - vec2<f32>(0.5, 0.5);
    let rotated_x = centered_uv.x * cos_angle + centered_uv.y * sin_angle;
    let wipe_coord = rotated_x + 0.5;
    
    if wipe_feather > 0.0 {
        let start_dist = wipe_coord - wipe_start;
        let end_dist = wipe_end - wipe_coord;
        return smoothstep(0.0, wipe_feather, start_dist) * smoothstep(0.0, wipe_feather, end_dist);
    } else {
        if wipe_coord < wipe_start || wipe_coord > wipe_end {
            return 0.0;
        }
        return 1.0;
    }
}

// Check if a point is inside a single mask shape (with rotation support)
// rotation: rotation angle in radians
// Returns true if inside the shape, false if outside
fn check_mask_shape_rotated(world_pos: vec2<f32>, mask_center: vec2<f32>, mask_half_size: vec2<f32>, is_ellipse: bool, rotation: f32) -> bool {
    // Transform world position to mask-local coordinates
    var rel_pos = world_pos - mask_center;
    
    // Apply inverse rotation to transform to axis-aligned space
    // rotation is the mask's rotation, so we rotate the point by -rotation
    if abs(rotation) > 0.001 {
        let cos_r = cos(-rotation);
        let sin_r = sin(-rotation);
        rel_pos = vec2<f32>(
            rel_pos.x * cos_r - rel_pos.y * sin_r,
            rel_pos.x * sin_r + rel_pos.y * cos_r
        );
    }
    
    if is_ellipse {
        // Ellipse equation: (x/a)^2 + (y/b)^2 <= 1
        let normalized = rel_pos / mask_half_size;
        return dot(normalized, normalized) <= 1.0;
    } else {
        // Rectangle mask (now axis-aligned after inverse rotation)
        return abs(rel_pos.x) <= mask_half_size.x && abs(rel_pos.y) <= mask_half_size.y;
    }
}

// Backward compatibility: check without rotation
fn check_mask_shape(world_pos: vec2<f32>, mask_center: vec2<f32>, mask_half_size: vec2<f32>, is_ellipse: bool) -> bool {
    return check_mask_shape_rotated(world_pos, mask_center, mask_half_size, is_ellipse, 0.0);
}

// Apply single mask - returns true if pixel should be kept
// mask_type: 0=disabled, 1=rect, 2=ellipse, 3=rect exclude, 4=ellipse exclude
fn apply_single_mask(world_pos: vec2<f32>, mask_type: f32, mask_center: vec2<f32>, mask_half_size: vec2<f32>) -> bool {
    // Disabled mask or invalid half_size - keep all pixels
    if mask_type < 0.5 || mask_half_size.x > 5000.0 {
        return true;
    }
    
    // Determine if this is an exclude mask (type >= 2.5)
    let is_exclude = mask_type > 2.5;
    // Determine if this is an ellipse (type ~= 2 or 4)
    let is_ellipse = (mask_type > 1.5 && mask_type < 2.5) || mask_type > 3.5;
    
    let inside = check_mask_shape(world_pos, mask_center, mask_half_size, is_ellipse);
    
    // For exclude masks, keep pixels OUTSIDE the shape
    if is_exclude {
        return !inside;
    }
    // For include masks, keep pixels INSIDE the shape
    return inside;
}

// Apply combined masks with correct AM logic:
// - Multiple include masks: INTERSECTION (show only where ALL include masks overlap)
// - Multiple exclude masks: UNION (hide if inside ANY exclude mask)
// - Mixed: (inside include intersection) AND (outside exclude union)
fn apply_masks(world_pos: vec2<f32>) -> bool {
    let mask1_type = effect_flags.x;
    let mask2_type = mask2_flags.x;
    // Rotation angles are stored in mask2_flags.y (mask1) and mask2_flags.z (mask2)
    let mask1_rotation = mask2_flags.y;
    let mask2_rotation = mask2_flags.z;
    
    // Disabled masks
    let mask1_enabled = mask1_type > 0.5;
    let mask2_enabled = mask2_type > 0.5;
    
    if !mask1_enabled && !mask2_enabled {
        return true; // No masks - keep all pixels
    }
    
    // Check if each mask is exclude type (type >= 2.5 means type 3 or 4)
    let mask1_is_exclude = mask1_type > 2.5;
    let mask2_is_exclude = mask2_type > 2.5;
    
    // Check if pixel is inside each mask shape (with rotation)
    let mask1_is_ellipse = (mask1_type > 1.5 && mask1_type < 2.5) || mask1_type > 3.5;
    let mask2_is_ellipse = (mask2_type > 1.5 && mask2_type < 2.5) || mask2_type > 3.5;
    
    let mask1_inside = mask1_enabled && mask_params.z < 5000.0 && 
        check_mask_shape_rotated(world_pos, mask_params.xy, mask_params.zw, mask1_is_ellipse, mask1_rotation);
    let mask2_inside = mask2_enabled && mask2_params.z < 5000.0 && 
        check_mask_shape_rotated(world_pos, mask2_params.xy, mask2_params.zw, mask2_is_ellipse, mask2_rotation);
    
    // Separate into include and exclude groups
    let include1 = mask1_enabled && !mask1_is_exclude;
    let include2 = mask2_enabled && !mask2_is_exclude;
    let exclude1 = mask1_enabled && mask1_is_exclude;
    let exclude2 = mask2_enabled && mask2_is_exclude;
    
    // Calculate include result: pixel must be inside ALL include masks (intersection)
    var include_pass = true;
    if include1 || include2 {
        if include1 && include2 {
            // Both include masks: must be inside both (intersection)
            include_pass = mask1_inside && mask2_inside;
        } else if include1 {
            include_pass = mask1_inside;
        } else {
            include_pass = mask2_inside;
        }
    }
    
    // Calculate exclude result: pixel must be outside ALL exclude masks
    var exclude_pass = true;
    if exclude1 || exclude2 {
        let in_exclude1 = exclude1 && mask1_inside;
        let in_exclude2 = exclude2 && mask2_inside;
        // If inside any exclude mask, fail
        exclude_pass = !(in_exclude1 || in_exclude2);
    }
    
    return include_pass && exclude_pass;
}

// Gaussian weight function
fn gaussian_weight(offset: f32, sigma: f32) -> f32 {
    return exp(-(offset * offset) / (2.0 * sigma * sigma));
}

// 2D Gaussian weight function
fn gaussian_weight_2d(dx: f32, dy: f32, sigma: f32) -> f32 {
    let d2 = dx * dx + dy * dy;
    return exp(-d2 / (2.0 * sigma * sigma));
}

// True 2D Gaussian blur with correct transparent boundary handling
// Boundary pixels outside [0,1] are treated as transparent (rgba(0,0,0,0))
// and participate in the weighted average to create proper edge fade-out
// blur_params: x = radius_px, y = orig_width, z = orig_height, w = expansion_px
fn apply_blur(uv: vec2<f32>) -> vec4<f32> {
    let radius = blur_params.x;
    let orig_width = blur_params.y;
    let orig_height = blur_params.z;
    
    // Pixel size in UV space
    let pixel_size_x = 1.0 / orig_width;
    let pixel_size_y = 1.0 / orig_height;
    
    // Sigma = radius / 2.0 for softer, more natural light diffusion (closer to Alight Motion)
    let sigma = max(radius / 2.0, 0.01);
    
    var total_color = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Sample radius covers 3*sigma for good distribution coverage
    // Cap at reasonable value for performance, but no step skipping to avoid artifacts
    let sample_radius = i32(min(ceil(sigma * 3.0), 64.0));
    
    // 2D grid sampling with Gaussian weights - no step skipping for quality
    for (var dy = -sample_radius; dy <= sample_radius; dy = dy + 1) {
        for (var dx = -sample_radius; dx <= sample_radius; dx = dx + 1) {
            let offset_x = f32(dx) * pixel_size_x;
            let offset_y = f32(dy) * pixel_size_y;
            let sample_uv = uv + vec2<f32>(offset_x, offset_y);
            
            // Calculate 2D Gaussian weight
            let weight = gaussian_weight_2d(f32(dx), f32(dy), sigma);
            
            // Skip negligible weights for performance
            if weight < 0.001 {
                continue;
            }
            
            // Sample color - treat out-of-bounds as transparent (rgba(0,0,0,0))
            // This is the key fix: boundary pixels participate in weighted average
            // with zero color contribution, causing proper edge fade-out
            var sample_color: vec4<f32>;
            if sample_uv.x >= 0.0 && sample_uv.x <= 1.0 && sample_uv.y >= 0.0 && sample_uv.y <= 1.0 {
                // Within bounds: normal sampling
                sample_color = textureSample(base_texture, base_sampler, sample_uv);
            } else {
                // Outside bounds: transparent black
                sample_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
            }
            
            // Always accumulate both color and weight
            total_color += sample_color * weight;
            total_weight += weight;
        }
    }
    
    // Normalize - with the fix above, total_weight should always be non-zero
    // for any UV that the 2D grid covers (which includes all mesh pixels)
    if total_weight > 0.0001 {
        return total_color / total_weight;
    } else {
        // Extreme edge case: return transparent
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

// Convert sRGB to linear color space (per channel)
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    } else {
        return pow((c + 0.055) / 1.055, 2.4);
    }
}

// Convert vec3 from sRGB to linear
fn srgb_to_linear_rgb(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgb_to_linear(c.r),
        srgb_to_linear(c.g),
        srgb_to_linear(c.b)
    );
}

// Get palette color by index (0-7)
// Palette colors are stored as sRGB values, so convert to linear for correct comparison
fn get_palette_color(index: i32) -> vec4<f32> {
    var color: vec4<f32>;
    switch(index) {
        case 0: { color = palette_color1; }
        case 1: { color = palette_color2; }
        case 2: { color = palette_color3; }
        case 3: { color = palette_color4; }
        case 4: { color = palette_color5; }
        case 5: { color = palette_color6; }
        case 6: { color = palette_color7; }
        case 7: { color = palette_color8; }
        default: { color = palette_color1; }
    }
    // Convert from sRGB to linear color space
    return vec4<f32>(srgb_to_linear_rgb(color.rgb), color.a);
}

// Calculate color distance with bias toward brighter palette colors
// This helps match AM's palette mapping behavior where brighter colors
// are preferred for pixels with moderate luminance
fn color_distance(c1: vec3<f32>, c2: vec3<f32>) -> f32 {
    let diff = c1 - c2;
    let dist = dot(diff, diff);
    
    // Calculate luminance of input color
    let input_lum = dot(c1, vec3<f32>(0.299, 0.587, 0.114));
    
    // If palette color is very dark (black), add penalty for bright inputs
    // This biases the algorithm toward selecting non-black colors for brighter pixels
    let palette_lum = dot(c2, vec3<f32>(0.299, 0.587, 0.114));
    if palette_lum < 0.01 && input_lum > 0.03 {
        return dist + input_lum * 2.5; // Add penalty proportional to input brightness
    }
    
    return dist;
}

// Apply palette map effect - quantize color to nearest palette color
fn apply_palette_map(input_color: vec4<f32>) -> vec4<f32> {
    let palette_count = i32(palette_flags.y);
    let shades_enabled = palette_flags.z > 0.5;
    
    // Extract RGB from input (keep alpha separate)
    let input_rgb = input_color.rgb;
    
    // Find nearest palette color
    var min_dist = 1000000.0;
    var nearest_index = 0;
    
    for (var i = 0; i < palette_count; i = i + 1) {
        let palette_rgb = get_palette_color(i).rgb;
        let dist = color_distance(input_rgb, palette_rgb);
        if dist < min_dist {
            min_dist = dist;
            nearest_index = i;
        }
    }
    
    let nearest_color = get_palette_color(nearest_index);
    
    // If shades enabled, blend based on luminance difference
    var result_rgb: vec3<f32>;
    if shades_enabled {
        // Calculate luminance of input and nearest palette color
        let input_lum = dot(input_rgb, vec3<f32>(0.299, 0.587, 0.114));
        let palette_lum = dot(nearest_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
        
        // Adjust palette color brightness to match input luminance
        let lum_ratio = input_lum / max(palette_lum, 0.001);
        result_rgb = nearest_color.rgb * clamp(lum_ratio, 0.0, 2.0);
    } else {
        result_rgb = nearest_color.rgb;
    }
    
    return vec4<f32>(result_rgb, input_color.a);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Extract effect flags
    // Mask is enabled if either mask1 or mask2 is enabled
    let mask_enabled = effect_flags.x > 0.5 || mask2_flags.x > 0.5;
    let wipe_enabled = effect_flags.y > 0.5;
    let stretch_enabled = effect_flags.z > 0.5;
    let blur_enabled = effect_flags.w > 0.5;
    let palette_enabled = palette_flags.x > 0.5;
    
    var sample_uv = mesh.uv;
    
    // Apply stretch segment effect if enabled (before blur)
    if stretch_enabled {
        sample_uv = apply_stretch_segment(mesh.uv);
        
        // Add small tolerance to prevent edge clipping due to floating point precision
        let eps = 0.002;
        if sample_uv.x < -eps || sample_uv.x > 1.0 + eps || sample_uv.y < -eps || sample_uv.y > 1.0 + eps {
            discard;
        }
        // Clamp to valid range for texture sampling
        sample_uv = clamp(sample_uv, vec2<f32>(0.0), vec2<f32>(1.0));
    }
    
    // Sample texture - with or without blur
    var tex_color: vec4<f32>;
    
    if blur_enabled {
        let blur_radius = blur_params.x;
        if blur_radius > 0.5 {
            tex_color = apply_blur(sample_uv);
        } else {
            tex_color = textureSample(base_texture, base_sampler, sample_uv);
        }
    } else {
        tex_color = textureSample(base_texture, base_sampler, sample_uv);
    }
    
    // Apply palette map effect if enabled
    if palette_enabled {
        let palette_alpha = palette_flags.w;
        let quantized_color = apply_palette_map(tex_color);
        // Blend between original and quantized based on palette alpha
        tex_color = mix(tex_color, quantized_color, palette_alpha);
    }
    
    // Apply mask clipping if any mask is enabled
    // Uses combined mask logic for dual masks and mixed include/exclude
    if mask_enabled {
        let world_pos = mesh.world_position.xy;
        if !apply_masks(world_pos) {
            discard;
        }
    }
    
    // Calculate wipe alpha if enabled
    var wipe_alpha = 1.0;
    if wipe_enabled {
        wipe_alpha = apply_wipe(mesh.uv);
        if wipe_alpha < 0.001 {
            discard;
        }
    }
    
    // Apply color tint and wipe alpha
    var final_color = tex_color * color;
    final_color.a *= wipe_alpha;
    
    if final_color.a < 0.001 {
        discard;
    }
    
    return final_color;
}
