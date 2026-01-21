// Stroked fill shader for Circle/Ellipse
// Uses sd_circle for circles (when a == b) and sd_ellipse for ellipses
// Uses fwidth-based anti-aliasing for proper rendering at any scale

let stroke_width = input.params.z;
// params.x = half_width (radius_x), params.y = half_height (radius_y)
let radius_x = input.params.x;
let radius_y = input.params.y;

// Minimum size threshold - skip rendering if shape is too small
// This prevents moire patterns and aliasing artifacts
let min_dimension = min(abs(radius_x), abs(radius_y));
if (min_dimension < 2.0) {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

// Use circle SDF when radii are equal (avoids division by zero in ellipse SDF)
// Use a small epsilon for floating point comparison
let is_circle = abs(radius_x - radius_y) < 0.001;
var dist: f32;
if (is_circle) {
    dist = smud::sd_circle(input.pos, radius_x);
} else {
    dist = smud::sd_ellipse(input.pos, radius_x, radius_y);
}

// Calculate adaptive anti-aliasing width based on distance field gradient
let aa_width = fwidth(dist);
let safe_aa_width = clamp(aa_width, 0.5, 10.0);

// Fill logic with adaptive AA
// Fill exists when dist < 0
let fill_alpha = 1.0 - smoothstep(-safe_aa_width, safe_aa_width, dist);
let fill_col = vec4<f32>(input.color.rgb, input.color.a * fill_alpha);

// Only process stroke if stroke_width > 0
if (stroke_width > 0.0) {
    // Unpack stroke color from params.w (stored as sRGB, need to convert to linear)
    let stroke_bits = bitcast<u32>(input.params.w);
    let stroke_r_srgb = f32((stroke_bits >> 24u) & 0xFFu) / 255.0;
    let stroke_g_srgb = f32((stroke_bits >> 16u) & 0xFFu) / 255.0;
    let stroke_b_srgb = f32((stroke_bits >> 8u) & 0xFFu) / 255.0;
    let stroke_a = f32(stroke_bits & 0xFFu) / 255.0;
    
    // Convert sRGB to linear (using gamma 2.2 approximation)
    let stroke_r = pow(stroke_r_srgb, 2.2);
    let stroke_g = pow(stroke_g_srgb, 2.2);
    let stroke_b = pow(stroke_b_srgb, 2.2);
    let stroke_color = vec4<f32>(stroke_r, stroke_g, stroke_b, stroke_a);

    // Stroke logic (Centered) with adaptive AA
    let half_stroke = stroke_width * 0.5;
    let dist_from_center_line = abs(dist);
    let stroke_alpha = 1.0 - smoothstep(half_stroke - safe_aa_width, half_stroke + safe_aa_width, dist_from_center_line);
    let stroke_col = vec4<f32>(stroke_color.rgb, stroke_color.a * stroke_alpha);

    // Composite: Stroke Over Fill
    let out_a = stroke_col.a + fill_col.a * (1.0 - stroke_col.a);

    if (out_a <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let out_rgb = (stroke_col.rgb * stroke_col.a + fill_col.rgb * fill_col.a * (1.0 - stroke_col.a)) / out_a;
    return vec4<f32>(out_rgb, out_a);
} else {
    // No stroke, just return fill
    if (fill_col.a <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return fill_col;
}
