// Black and White Tilemap Shader for bevy_ecs_tilemap
// bevy_ecs_tilemap 的黑白瓦片地图着色器
//
// Converts tilemap textures to black and white with fade control and pixel noise.
// in.color.r is used as fade value:
//   0.0 = pure black, 0.5 = normal B&W, 1.0 = pure white
// During transition, tile pixels randomly "generate" based on noise.
// 将瓦片地图纹理转换为带淡入控制和像素噪点的黑白色。
// in.color.r 用作淡入值：
//   0.0 = 纯黑，0.5 = 正常黑白，1.0 = 纯白
// 过渡期间，瓦片像素根据噪点随机"生成"。

#import bevy_ecs_tilemap::common::{tilemap_data, sprite_texture, sprite_sampler}
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

// ========== BLACK/WHITE THRESHOLD ==========
// Lower value = more white, higher value = more black.
// 较低的值 = 更多白色，较高的值 = 更多黑色。
const THRESHOLD: f32 = 0.125;

// Simple hash function for pixel noise
// 用于像素噪点的简单哈希函数
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

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

    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset);
    #else
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv.xy, in.tile_id);
    #endif
    
    // Discard fully transparent pixels
    // 丢弃纯透明的像素
    if (tex_color.a < 0.01) {
        discard;
    }
    
    // Fade value from TileColor.r: 0.0=black, 0.5=normal, 1.0=white
    // 从 TileColor.r 获取淡入值：0.0=黑，0.5=正常，1.0=白
    let fade = in.color.r;
    
    // Convert to grayscale using luminance formula
    // 使用亮度公式转换为灰度
    let luminance = dot(tex_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // Threshold to pure black or white (the normal B&W result)
    // 阈值化为纯黑或纯白（正常的黑白结果）
    let bw = select(0.0, 1.0, luminance > THRESHOLD);
    
    // Calculate tile pixel position (aligned to tile's pixel grid)
    // in.uv.zw is 0-1 within the tile, multiply by tile_size to get pixel coords
    // Floor to get discrete pixel position
    // 计算瓦片像素位置（对齐到瓦片的像素网格）
    // in.uv.zw 在瓦片内是 0-1，乘以 tile_size 得到像素坐标
    // 向下取整得到离散像素位置
    let tile_pixel = floor(in.uv.zw * tilemap_data.tile_size);
    
    // Combine with tile grid position for unique per-pixel value across the map
    // 与瓦片网格位置组合，得到整个地图中每个像素的唯一值
    let world_pixel = vec2<f32>(f32(in.storage_position.x), f32(in.storage_position.y)) * tilemap_data.tile_size + tile_pixel;
    let noise = hash21(world_pixel);
    
    // Apply fade with pixel-level noise:
    // fade > 0.5: transitioning from white to normal B&W
    //   - pixels with noise < progress show their B&W value
    //   - other pixels stay white
    // fade <= 0.5: normal B&W (no noise effect)
    // 使用像素级噪点应用淡入：
    // fade > 0.5：从白色过渡到正常黑白
    //   - noise < progress 的像素显示其黑白值
    //   - 其他像素保持白色
    // fade <= 0.5：正常黑白（无噪点效果）
    var final_color: f32;
    if (fade > 0.5) {
        // Progress from 0 (at fade=1.0) to 1 (at fade=0.5)
        // 从 0（fade=1.0时）到 1（fade=0.5时）的进度
        let progress = (1.0 - fade) * 2.0;
        
        // Pixel shows B&W if noise < progress, otherwise white
        // 如果 noise < progress 则像素显示黑白，否则显示白色
        if (noise < progress) {
            final_color = bw;
        } else {
            final_color = 1.0;
        }
    } else {
        // Normal B&W rendering
        // 正常黑白渲染
        final_color = bw;
    }
    
    return vec4<f32>(final_color, final_color, final_color, tex_color.a);
}
