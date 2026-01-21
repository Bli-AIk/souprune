// Stroked fill shader for Box (Rect)
// Explicitly calculates distance using sd_box
// Uses fwidth-based anti-aliasing for proper rendering at any scale

let stroke_width = input.params.z;
// Calculate distance explicitly for Box
// params.x = half_width, params.y = half_height
let dist = smud::sd_box(input.pos, vec2<f32>(input.params.x, input.params.y));

// Calculate adaptive anti-aliasing width based on distance field gradient
// fwidth gives us the rate of change of dist across neighboring pixels
let aa_width = fwidth(dist);
// Clamp aa_width to avoid division by zero or too thin AA
let safe_aa_width = max(aa_width, 0.001);

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
// Stroke band covers distance [-half_stroke, +half_stroke]
let half_stroke = stroke_width * 0.5;
let dist_from_center_line = abs(dist);
let stroke_alpha = 1.0 - smoothstep(half_stroke - safe_aa_width, half_stroke + safe_aa_width, dist_from_center_line);
let stroke_col = vec4<f32>(stroke_color.rgb, stroke_color.a * stroke_alpha);

// Fill logic with adaptive AA
// Fill exists when dist < 0
let fill_alpha = 1.0 - smoothstep(-safe_aa_width, safe_aa_width, dist);
let fill_col = vec4<f32>(input.color.rgb, input.color.a * fill_alpha);

// Composite: Stroke Over Fill
let out_a = stroke_col.a + fill_col.a * (1.0 - stroke_col.a);

if (out_a <= 0.0) {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

let out_rgb = (stroke_col.rgb * stroke_col.a + fill_col.rgb * fill_col.a * (1.0 - stroke_col.a)) / out_a;

return vec4<f32>(out_rgb, out_a);
