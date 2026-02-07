// Enemy HP Bar Sprite Shader - Based on player HP bar shader
// 敌人 HP 条精灵着色器 - 基于玩家 HP 条着色器
//
// This shader displays enemy HP with a green color scheme (no lag effect).
// 此着色器使用绿色配色方案显示敌人 HP（无延迟效果）。
//
// Uniform data passed via color_params (alphabetical order):
// - r: alpha (params sorted alphabetically: alpha, half_width, hp_ratio, lag_ratio)
// - g: half_width (unused)
// - b: hp_ratio - Current HP percentage (0.0-1.0)
// - a: lag_ratio (unused for enemy HP)

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
    
    // DEBUG: Visualize hp_ratio directly as color intensity
    // If all bars show the same color, they're getting the same hp_ratio value
    // hp_ratio=1.0 -> bright green, hp_ratio=0.5 -> medium green
    
    // UV.x goes from 0 (left) to 1 (right)
    let t = in.uv.x;
    
    // Enemy HP bar colors (green theme)
    let col_bg = vec3<f32>(1.0, 0.0, 0.0);      // Pure red background (empty HP)
    let col_hp = vec3<f32>(0.0, 1.0, 0.0);      // Green current HP
    
    // Layer logic: green > red
    var final_color: vec3<f32>;
    if (t < hp_ratio) {
        final_color = col_hp;
    } else {
        final_color = col_bg;
    }
    
    return vec4<f32>(final_color, alpha);
}
