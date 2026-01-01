// Black and White Tile Shader
// 黑白瓦片着色器
//
// Converts tile textures to pure black and white (only #000000 and #FFFFFF).
// White is the primary color, black is used for darker areas only.
// 将瓦片纹理转换为纯黑白色（仅包含 #000000 和 #FFFFFF）。
// 白色为主，黑色仅用于较暗的区域。

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// ========== BLACK/WHITE THRESHOLD ==========
// IMPORTANT: This value MUST match the THRESHOLD in black_white_tilemap.wgsl!
// To change, update BOTH this file AND black_white_tilemap.wgsl.
// Lower value = more white, higher value = more black.
// 重要：此值必须与 black_white_tilemap.wgsl 中的 THRESHOLD 匹配！
// 要更改，请同时更新此文件和 black_white_tilemap.wgsl。
// 较低的值 = 更多白色，较高的值 = 更多黑色。
const THRESHOLD: f32 = 0.125; // <-- MUST MATCH black_white_tilemap.wgsl / 必须与 black_white_tilemap.wgsl 匹配

@group(2) @binding(0)
var<uniform> _threshold: f32; // Unused, kept for compatibility / 未使用，保留兼容性

@group(2) @binding(1)
var base_texture: texture_2d<f32>;

@group(2) @binding(2)
var base_sampler: sampler;

@group(2) @binding(3)
var<uniform> uv_rect: vec4<f32>;

// Simple hash for per-pixel randomness
// 用于每像素随机性的简单哈希
fn hash21_tile(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Map UV from mesh space to texture atlas space
    // 将 UV 从网格空间映射到纹理图集空间
    let atlas_uv = vec2<f32>(
        uv_rect.x + in.uv.x * uv_rect.z,
        uv_rect.y + in.uv.y * uv_rect.w
    );
    
    let tex_color = textureSample(base_texture, base_sampler, atlas_uv);
    
    // Discard fully transparent pixels (alpha == 0)
    // 丢弃纯透明的像素（alpha == 0）
    if tex_color.a < 0.01 {
        discard;
    }
    
    // Convert to grayscale using luminance formula
    // 使用亮度公式转换为灰度
    let luminance = dot(tex_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // Threshold to pure black or white
    // Pixels with luminance > THRESHOLD become WHITE, otherwise BLACK
    // 阈值化为纯黑或纯白
    // 亮度 > THRESHOLD 的像素变为白色，否则为黑色
    let bw = select(0.0, 1.0, luminance > THRESHOLD);
    
    // ========== PIXEL GENERATION ANIMATION ==========
    // Calculate pixel position in texture space for stable randomness
    // 计算纹理空间中的像素位置以获得稳定的随机性
    let pixel_pos = atlas_uv * vec2<f32>(1024.0, 1024.0); // Approximate texture size
    let pixel_random = hash21_tile(floor(pixel_pos));
    
    // Use vertex color red channel as animation progress (if available)
    // Assume in.color.r ranges from 0.0 (early) to 1.0 (late)
    // For now, default to fully visible (1.0) since tile shader may not have time uniform
    // 使用顶点颜色红色通道作为动画进度（如果可用）
    // 假设 in.color.r 范围从 0.0（早期）到 1.0（晚期）
    // 目前，默认为完全可见 (1.0)，因为瓦片着色器可能没有时间 uniform
    let anim_progress = in.color.r;
    
    // Calculate pixel generation timing
    // 计算像素生成时机
    let pixel_start = pixel_random;
    let pixel_progress = clamp((anim_progress - pixel_start) * 2.0, 0.0, 1.0);
    
    // Apply ease-out cubic easing
    // 应用 ease-out 三次方缓动
    let eased_progress = 1.0 - pow(1.0 - pixel_progress, 3.0);
    
    return vec4<f32>(bw, bw, bw, tex_color.a * eased_progress);
}
