// Black and White Tilemap Shader for bevy_ecs_tilemap
// bevy_ecs_tilemap 的黑白瓦片地图着色器
//
// Converts tilemap textures to pure black and white (only #000000 and #FFFFFF).
// White is the primary color, black is used for darker areas only.
// 将瓦片地图纹理转换为纯黑白色（仅包含 #000000 和 #FFFFFF）。
// 白色为主，黑色仅用于较暗的区域。

#import bevy_ecs_tilemap::common::{tilemap_data, sprite_texture, sprite_sampler}
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

// ========== BLACK/WHITE THRESHOLD ==========
// IMPORTANT: This value MUST match the THRESHOLD in black_white_tile.wgsl!
// To change, update BOTH this file AND black_white_tile.wgsl.
// Lower value = more white, higher value = more black.
// 重要：此值必须与 black_white_tile.wgsl 中的 THRESHOLD 匹配！
// 要更改，请同时更新此文件和 black_white_tile.wgsl。
// 较低的值 = 更多白色，较高的值 = 更多黑色。
const THRESHOLD: f32 = 0.125; // <-- MUST MATCH black_white_tile.wgsl / 必须与 black_white_tile.wgsl 匹配

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    #ifdef ATLAS
    let half_texture_pixel_size_u = 0.5 / tilemap_data.texture_size.x;
    let half_texture_pixel_size_v = 0.5 / tilemap_data.texture_size.y;
    let half_tile_pixel_size_u = 0.5 / tilemap_data.tile_size.x;
    let half_tile_pixel_size_v = 0.5 / tilemap_data.tile_size.y;

    // Offset the UV 1/2 pixel from the sides of the tile
    var uv_offset: vec2<f32> = vec2<f32>(0.0, 0.0);
    if (in.uv.z < half_tile_pixel_size_u) {
        uv_offset.x = half_texture_pixel_size_u;
    } else if (in.uv.z > (1.0 - half_tile_pixel_size_u)) {
        uv_offset.x = - half_texture_pixel_size_u;
    }
    if (in.uv.w < half_tile_pixel_size_v) {
        uv_offset.y = half_texture_pixel_size_v;
    } else if (in.uv.w > (1.0 - half_tile_pixel_size_v)) {
        uv_offset.y = - half_texture_pixel_size_v;
    }

    // Sample texture WITHOUT multiplying by in.color to match sprite shader behavior
    // 采样纹理时不乘以 in.color，以匹配精灵着色器的行为
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset);
    #else
    // Sample texture WITHOUT multiplying by in.color to match sprite shader behavior
    // 采样纹理时不乘以 in.color，以匹配精灵着色器的行为
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv.xy, in.tile_id);
    #endif
    
    // Discard fully transparent pixels (alpha == 0)
    // 丢弃纯透明的像素（alpha == 0）
    if (tex_color.a < 0.01) {
        discard;
    }
    
    // Convert to grayscale using luminance formula
    // 使用亮度公式转换为灰度
    let luminance = dot(tex_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // Threshold to pure black or white
    // 阈值化为纯黑或纯白
    let bw = select(0.0, 1.0, luminance > THRESHOLD);
    
    return vec4<f32>(bw, bw, bw, tex_color.a);
}
