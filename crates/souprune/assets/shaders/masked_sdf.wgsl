// Masked SDF shader - SDF shapes with mask support
//
// This shader renders SDF shapes (circle, rect) with optional mask clipping.
//
// Uniform layout:
// 0: color (vec4) - fill color
// 1: params (vec4) - (half_width, half_height, stroke_width, packed_stroke_color)
// 2: mask_params (vec4) - (center_x, center_y, half_width, half_height)
// 3: flags (vec4) - (shape_type, mask_type, 0, 0)
//    - shape_type: 0 = circle, 1 = rect
//    - mask_type: 0 = no mask, 1 = rect mask, 2 = ellipse mask

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> params: vec4<f32>;
@group(2) @binding(2) var<uniform> mask_params: vec4<f32>;
@group(2) @binding(3) var<uniform> flags: vec4<f32>;

// SDF for circle
fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// SDF for box (rectangle)
fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

// Apply mask clipping - returns true if pixel should be kept
// mask_type: 1.0 = rectangle, 2.0 = ellipse, 3.0 = rectangle exclude, 4.0 = ellipse exclude
fn apply_mask(world_pos: vec2<f32>) -> bool {
    let mask_center = mask_params.xy;
    let mask_half_size = mask_params.zw;
    
    // No mask if half_size is very large (default value)
    if mask_half_size.x > 5000.0 {
        return true;
    }
    
    let rel_pos = world_pos - mask_center;
    let mask_type = flags.y;
    
    // Determine if this is an exclude mask (mask_type >= 2.5)
    let is_exclude = mask_type > 2.5;
    // Determine if this is an ellipse (mask_type ~= 2 or 4)
    let is_ellipse = (mask_type > 1.5 && mask_type < 2.5) || mask_type > 3.5;
    
    var inside: bool;
    if is_ellipse {
        // Ellipse equation: (x/a)^2 + (y/b)^2 <= 1
        let nx = rel_pos.x / mask_half_size.x;
        let ny = rel_pos.y / mask_half_size.y;
        inside = (nx * nx + ny * ny) <= 1.0;
    } else {
        // Rectangle mask
        inside = abs(rel_pos.x) <= mask_half_size.x && abs(rel_pos.y) <= mask_half_size.y;
    }
    
    // For exclude masks, we want to keep pixels OUTSIDE the mask
    if is_exclude {
        return !inside;
    }
    return inside;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Get parameters
    let half_width = params.x;
    let half_height = params.y;
    let stroke_width = params.z;
    let packed_stroke = bitcast<u32>(params.w);
    
    let shape_type = flags.x;
    let mask_type = flags.y;
    
    // Convert UV to shape space position
    // UV is [0,1], convert to centered coordinates scaled by shape size
    // Assuming mesh is sized to match shape bounds
    let pos = (mesh.uv - vec2<f32>(0.5)) * vec2<f32>(half_width, half_height) * 2.0;
    
    // Calculate distance based on shape type
    var dist: f32;
    if shape_type < 0.5 {
        // Circle: use min(half_width, half_height) as radius
        let radius = min(half_width, half_height);
        dist = sd_circle(pos, radius);
    } else {
        // Rectangle
        dist = sd_box(pos, vec2<f32>(half_width, half_height));
    }
    
    // Anti-aliasing
    let aa_width = fwidth(dist);
    let safe_aa_width = max(aa_width, 0.001);
    
    // Unpack stroke color
    let stroke_r_srgb = f32((packed_stroke >> 24u) & 0xFFu) / 255.0;
    let stroke_g_srgb = f32((packed_stroke >> 16u) & 0xFFu) / 255.0;
    let stroke_b_srgb = f32((packed_stroke >> 8u) & 0xFFu) / 255.0;
    let stroke_a = f32(packed_stroke & 0xFFu) / 255.0;
    
    // Convert sRGB to linear
    let stroke_r = pow(stroke_r_srgb, 2.2);
    let stroke_g = pow(stroke_g_srgb, 2.2);
    let stroke_b = pow(stroke_b_srgb, 2.2);
    let stroke_color = vec4<f32>(stroke_r, stroke_g, stroke_b, stroke_a);
    
    // Stroke logic (Centered)
    let half_stroke = stroke_width * 0.5;
    let dist_from_center_line = abs(dist);
    let stroke_alpha = 1.0 - smoothstep(half_stroke - safe_aa_width, half_stroke + safe_aa_width, dist_from_center_line);
    let stroke_col = vec4<f32>(stroke_color.rgb, stroke_color.a * stroke_alpha);
    
    // Fill logic
    let fill_alpha = 1.0 - smoothstep(-safe_aa_width, safe_aa_width, dist);
    let fill_col = vec4<f32>(color.rgb, color.a * fill_alpha);
    
    // Composite: Stroke Over Fill
    let out_a = stroke_col.a + fill_col.a * (1.0 - stroke_col.a);
    
    if out_a <= 0.0 {
        discard;
    }
    
    let out_rgb = (stroke_col.rgb * stroke_col.a + fill_col.rgb * fill_col.a * (1.0 - stroke_col.a)) / out_a;
    var final_color = vec4<f32>(out_rgb, out_a);
    
    // Apply mask if enabled
    if mask_type > 0.5 {
        let world_pos = mesh.world_position.xy;
        if !apply_mask(world_pos) {
            discard;
        }
    }
    
    return final_color;
}
