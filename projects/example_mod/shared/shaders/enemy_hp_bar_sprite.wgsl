// Enemy HP Bar Sprite Shader - Based on player HP bar shader
// 敌人 HP 条精灵着色器 - 基于玩家 HP 条着色器
//
// This shader displays enemy HP with a green color scheme (no lag effect).
// 此着色器使用绿色配色方案显示敌人 HP（无延迟效果）。
//
// Uniform data passed via color_params:
// - r: Current HP percentage (0.0-1.0)
// - g: (unused, can be used for effects)
// - b: (unused)
// - a: Alpha

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> color_params: vec4<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var base_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let hp_ratio = color_params.r;
    
    // UV.x goes from 0 (left) to 1 (right)
    let t = in.uv.x;
    
    // Enemy HP bar colors (green theme)
    let col_bg = vec3<f32>(0.2, 0.0, 0.0);      // Dark red background (empty HP)
    let col_hp = vec3<f32>(0.0, 1.0, 0.0);      // Green current HP
    
    // Layer logic: green > dark red
    var final_color: vec3<f32>;
    if (t < hp_ratio) {
        final_color = col_hp;
    } else {
        final_color = col_bg;
    }
    
    return vec4<f32>(final_color, color_params.a);
}
