// Black and White Tile Shader
// 黑白瓦片着色器
//
// Converts tile textures to pure black and white (only #000000 and #FFFFFF).
// White is the primary color, black is used for darker areas only.
// 将瓦片纹理转换为纯黑白色（仅包含 #000000 和 #FFFFFF）。
// 白色为主，黑色仅用于较暗的区域。

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> threshold: f32;

@group(2) @binding(1)
var base_texture: texture_2d<f32>;

@group(2) @binding(2)
var base_sampler: sampler;

@group(2) @binding(3)
var<uniform> uv_rect: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Map UV from mesh space to texture atlas space
    // 将 UV 从网格空间映射到纹理图集空间
    let atlas_uv = vec2<f32>(
        uv_rect.x + in.uv.x * uv_rect.z,
        uv_rect.y + in.uv.y * uv_rect.w
    );
    
    let tex_color = textureSample(base_texture, base_sampler, atlas_uv);
    
    // Discard fully transparent pixels
    // 丢弃完全透明的像素
    if tex_color.a < 0.01 {
        discard;
    }
    
    // Convert to grayscale using luminance formula
    // 使用亮度公式转换为灰度
    let luminance = dot(tex_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // Threshold to pure black or white
    // Lower threshold means more white (luminance > threshold -> white)
    // 阈值化为纯黑或纯白
    // 较低的阈值意味着更多白色（亮度 > 阈值 -> 白色）
    let bw = select(0.0, 1.0, luminance > threshold);
    
    return vec4<f32>(bw, bw, bw, tex_color.a);
}
