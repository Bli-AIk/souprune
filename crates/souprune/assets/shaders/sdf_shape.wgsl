// SDF Shape Shader - Custom implementation for Bevy
//
// Renders rectangles (round/miter/bevel corners) and circles/ellipses
// with optional strokes using signed distance field techniques.
//
// Uniforms (in SdfMaterialUniform struct):
// - color: Fill color (vec4<f32>)
// - params: (half_width, half_height, stroke_width, packed_stroke_color)
// - mask_params: (mask_center_x, mask_center_y, mask_half_width, mask_half_height)
// - mask2_params: (mask2_center_x, mask2_center_y, mask2_half_width, mask2_half_height)
// - shape_type: 0=BoxRound, 1=BoxMiter, 2=BoxBevel, 3=Circle
// - mask_type: 0=disabled, 1=rectangle, 2=ellipse, 3=rect exclude, 4=ellipse exclude
// - mask2_type: 0=disabled, 1=rectangle, 2=ellipse, 3=rect exclude, 4=ellipse exclude
// - frame_half: half of mesh quad size
// - mask_rotation: rotation of mask 1 in radians
// - mask2_rotation: rotation of mask 2 in radians

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SdfMaterialUniform {
    color: vec4<f32>,
    params: vec4<f32>,
    mask_params: vec4<f32>,
    mask2_params: vec4<f32>,
    shape_type: f32,
    mask_type: f32,
    mask2_type: f32,
    frame_half: f32,
    mask_rotation: f32,
    mask2_rotation: f32,
    _padding1: f32,
    _padding2: f32,
};

@group(2) @binding(0) var<uniform> material: SdfMaterialUniform;

// SDF for rounded box (Euclidean distance)
fn sd_box(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let d = abs(p) - half_size;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

// SDF for sharp box with miter corners (Chebyshev/L-infinity)
fn sd_box_miter(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let d = abs(p) - half_size;
    return max(d.x, d.y);
}

// SDF for box with bevel corners
fn sd_box_bevel(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    let d = abs(p) - half_size;
    let dist_miter = max(d.x, d.y);
    let dist_bevel = d.x + d.y;
    return max(dist_miter, dist_bevel);
}

// SDF for circle
fn sd_circle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

// SDF for ellipse (approximate)
fn sd_ellipse(p: vec2<f32>, a: f32, b: f32) -> f32 {
    // Normalize to unit circle space, compute distance, scale back
    let q = p / vec2<f32>(a, b);
    let d = length(q) - 1.0;
    // Scale distance back (approximate)
    let scale = min(a, b);
    return d * scale;
}

// Unpack RGBA from u32 bits stored in f32
fn unpack_color(packed: f32) -> vec4<f32> {
    let bits = bitcast<u32>(packed);
    let r_srgb = f32((bits >> 24u) & 0xFFu) / 255.0;
    let g_srgb = f32((bits >> 16u) & 0xFFu) / 255.0;
    let b_srgb = f32((bits >> 8u) & 0xFFu) / 255.0;
    let a = f32(bits & 0xFFu) / 255.0;
    
    // Convert sRGB to linear (gamma 2.2 approximation)
    let r = pow(r_srgb, 2.2);
    let g = pow(g_srgb, 2.2);
    let b = pow(b_srgb, 2.2);
    
    return vec4<f32>(r, g, b, a);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Check dual-mask system - use world position for mask testing
    // mask_type: 1.0 = rectangle include, 2.0 = ellipse include, 
    //            3.0 = rectangle exclude, 4.0 = ellipse exclude
    let mask1_type = material.mask_type;
    let mask2_type = material.mask2_type;
    
    let mask1_enabled = mask1_type > 0.5 && material.mask_params.z < 5000.0;
    let mask2_enabled = mask2_type > 0.5 && material.mask2_params.z < 5000.0;
    
    // For include masks: can do early discard outside mask area
    // For exclude masks: must NOT do early discard (we keep pixels OUTSIDE the mask)
    let mask1_is_include_type = mask1_type > 0.5 && mask1_type < 2.5; // types 1.0 and 2.0 are includes
    if mask1_enabled && mask1_is_include_type {
        let world_pos = in.world_position.xy;
        let center1 = material.mask_params.xy;
        let half_size1 = material.mask_params.zw;
        var rel_pos1 = world_pos - center1;
        
        // Apply rotation
        let rot1 = -material.mask_rotation;
        let cos1 = cos(rot1);
        let sin1 = sin(rot1);
        rel_pos1 = vec2<f32>(
            rel_pos1.x * cos1 - rel_pos1.y * sin1,
            rel_pos1.x * sin1 + rel_pos1.y * cos1
        );
        
        let is_ellipse1 = mask1_type > 1.5;
        var inside: bool;
        if is_ellipse1 {
            let normalized1 = rel_pos1 / half_size1;
            inside = dot(normalized1, normalized1) <= 1.0;
        } else {
            inside = abs(rel_pos1.x) <= half_size1.x && abs(rel_pos1.y) <= half_size1.y;
        }
        
        if !inside {
            // Outside include mask - discard
            discard;
        }
        // Inside include mask - continue to normal rendering below
    }
    
    if mask1_enabled || mask2_enabled {
        let world_pos = in.world_position.xy;
        
        // Helper: check if point is inside a mask (with rotation support)
        // Returns true if inside, false if outside
        var mask1_inside = true;  // Default: all pixels pass
        var mask1_is_exclude = false;
        if mask1_enabled {
            let center1 = material.mask_params.xy;
            let half_size1 = material.mask_params.zw;
            var rel_pos1 = world_pos - center1;
            
            // Apply rotation (rotate point by -rotation to transform to mask's local space)
            let rot1 = -material.mask_rotation;
            let cos1 = cos(rot1);
            let sin1 = sin(rot1);
            rel_pos1 = vec2<f32>(
                rel_pos1.x * cos1 - rel_pos1.y * sin1,
                rel_pos1.x * sin1 + rel_pos1.y * cos1
            );
            
            mask1_is_exclude = mask1_type > 2.5;
            let is_ellipse1 = (mask1_type > 1.5 && mask1_type < 2.5) || mask1_type > 3.5;
            
            if is_ellipse1 {
                let normalized1 = rel_pos1 / half_size1;
                mask1_inside = dot(normalized1, normalized1) <= 1.0;
            } else {
                mask1_inside = abs(rel_pos1.x) <= half_size1.x && abs(rel_pos1.y) <= half_size1.y;
            }
        }
        
        var mask2_inside = true;  // Default: all pixels pass  
        var mask2_is_exclude = false;
        if mask2_enabled {
            let center2 = material.mask2_params.xy;
            let half_size2 = material.mask2_params.zw;
            var rel_pos2 = world_pos - center2;
            
            // Apply rotation (rotate point by -rotation to transform to mask's local space)
            let rot2 = -material.mask2_rotation;
            let cos2 = cos(rot2);
            let sin2 = sin(rot2);
            rel_pos2 = vec2<f32>(
                rel_pos2.x * cos2 - rel_pos2.y * sin2,
                rel_pos2.x * sin2 + rel_pos2.y * cos2
            );
            
            mask2_is_exclude = mask2_type > 2.5;
            let is_ellipse2 = (mask2_type > 1.5 && mask2_type < 2.5) || mask2_type > 3.5;
            
            if is_ellipse2 {
                let normalized2 = rel_pos2 / half_size2;
                mask2_inside = dot(normalized2, normalized2) <= 1.0;
            } else {
                mask2_inside = abs(rel_pos2.x) <= half_size2.x && abs(rel_pos2.y) <= half_size2.y;
            }
        }
        
        // Apply dual-mask logic:
        // - Include masks: intersection (AND) - pixel must be inside ALL includes
        // - Exclude masks: union (OR) - pixel must be outside ALL excludes  
        // - Mixed: (inside all includes) AND (outside all excludes)
        
        var should_discard = false;
        
        // Count include and exclude masks
        let mask1_is_include = mask1_enabled && !mask1_is_exclude;
        let mask2_is_include = mask2_enabled && !mask2_is_exclude;
        let has_includes = mask1_is_include || mask2_is_include;
        let has_excludes = (mask1_enabled && mask1_is_exclude) || (mask2_enabled && mask2_is_exclude);
        
        if has_includes {
            // For includes: use intersection (AND) - must be inside ALL include masks
            var include_result = true;
            if mask1_is_include {
                include_result = include_result && mask1_inside;
            }
            if mask2_is_include {
                include_result = include_result && mask2_inside;
            }
            if !include_result {
                should_discard = true;
            }
        }
        
        if has_excludes && !should_discard {
            // For excludes: use union (OR) - must be outside ANY exclude mask (discard if inside any)
            if (mask1_enabled && mask1_is_exclude && mask1_inside) || 
               (mask2_enabled && mask2_is_exclude && mask2_inside) {
                should_discard = true;
            }
        }
        
        if should_discard {
            discard;
        }
    }

    let half_width = material.params.x;
    let half_height = material.params.y;
    let stroke_width = material.params.z;
    let packed_stroke = material.params.w;
    
    // The mesh is created with size = frame_half * 2 x frame_half * 2
    // UV (0,0) is bottom-left, (1,1) is top-right
    // We need position relative to center
    
    // Use the frame_half stored in the material (computed at spawn time)
    let frame_size = material.frame_half * 2.0;
    
    // Convert UV to local coordinates centered at origin
    let pos = (in.uv - 0.5) * frame_size;
    
    // Calculate SDF based on shape type
    var dist: f32;
    let shape_type = i32(material.shape_type);
    
    if shape_type == 0 {
        // BoxRound
        dist = sd_box(pos, vec2<f32>(half_width, half_height));
    } else if shape_type == 1 {
        // BoxMiter
        dist = sd_box_miter(pos, vec2<f32>(half_width, half_height));
    } else if shape_type == 2 {
        // BoxBevel
        dist = sd_box_bevel(pos, vec2<f32>(half_width, half_height));
    } else {
        // Circle/Ellipse
        if abs(half_width - half_height) < 0.001 {
            dist = sd_circle(pos, half_width);
        } else {
            dist = sd_ellipse(pos, half_width, half_height);
        }
    }
    
    // Hard edge rendering (pixel-perfect for retro style games)
    // Use step instead of smoothstep for crisp edges
    
    // Fill: inside the shape (dist <= 0)
    let fill_alpha = step(dist, 0.0);
    let fill_col = vec4<f32>(material.color.rgb, material.color.a * fill_alpha);
    
    // Handle stroke if stroke_width > 0
    if stroke_width > 0.0 {
        let stroke_color = unpack_color(packed_stroke);
        
        // Centered stroke: distance band around the edge
        let half_stroke = stroke_width * 0.5;
        let dist_from_edge = abs(dist);
        // Hard edge stroke: inside if dist_from_edge <= half_stroke
        let stroke_alpha = step(dist_from_edge, half_stroke);
        let stroke_col = vec4<f32>(stroke_color.rgb, stroke_color.a * stroke_alpha);
        
        // Composite: stroke over fill
        let out_a = stroke_col.a + fill_col.a * (1.0 - stroke_col.a);
        
        // Use threshold to handle floating point precision issues
        if out_a < 0.01 {
            discard;
        }
        
        let out_rgb = (stroke_col.rgb * stroke_col.a + fill_col.rgb * fill_col.a * (1.0 - stroke_col.a)) / out_a;
        return vec4<f32>(out_rgb, out_a);
    } else {
        // No stroke, just fill
        if fill_col.a <= 0.0 {
            discard;
        }
        return fill_col;
    }
}
