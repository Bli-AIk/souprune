// HP Bar Sprite Shader - Undertale style with horizontal gradient
// HP 条精灵着色器 - Undertale 风格的水平渐变
//
// This shader is applied to a sprite (not SDF) so we have UV coordinates!
// 此着色器应用于精灵（非 SDF），所以我们有 UV 坐标！
//
// Uniform data passed via color_params (alphabetical order):
// - r: alpha (params sorted alphabetically: alpha, half_width, hp_ratio, lag_ratio)
// - g: half_width (not used, UV handles positioning)
// - b: hp_ratio - Current HP percentage (0.0-1.0)
// - a: lag_ratio - Delayed HP percentage (0.0-1.0) for white bar effect

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> color_params: vec4<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var base_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Parameters are sorted alphabetically: alpha, half_width, hp_ratio, lag_ratio
    let alpha = color_params.r;
    let hp_ratio = color_params.b;
    let lag_ratio = color_params.a;
    
    // UV.x goes from 0 (left) to 1 (right)
    let t = in.uv.x;
    
    // Undertale HP bar colors
    let col_bg = vec3<f32>(1.0, 0.0, 0.0);      // Red background (empty HP)
    let col_lag = vec3<f32>(1.0, 1.0, 1.0);     // White delayed bar
    let col_hp = vec3<f32>(1.0, 1.0, 0.0);      // Yellow current HP
    
    // Layer logic: yellow > white > red
    var final_color: vec3<f32>;
    if (t < hp_ratio) {
        final_color = col_hp;
    } else if (t < lag_ratio) {
        final_color = col_lag;
    } else {
        final_color = col_bg;
    }
    
    return vec4<f32>(final_color, alpha);
}
