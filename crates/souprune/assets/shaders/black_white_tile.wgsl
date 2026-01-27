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
    // Smooth, continuous alpha-based animation without any stepping
    // 完全连续的基于alpha的动画，无任何步进
    
    // Calculate pixel position for stable per-pixel randomness
    // 计算像素位置以获得稳定的每像素随机性
    let pixel_pos = atlas_uv * vec2<f32>(1024.0, 1024.0);
    let pixel_random = hash21_tile(floor(pixel_pos));
    
    // Global animation progress with extended range
    // 全局动画进度，扩展范围
    let global_progress = clamp(in.color.r * 1.5, 0.0, 1.0);
    
    // Smaller offset range (0-0.3) for smoother, more continuous flow
    // 更小的偏移范围（0-0.3）以实现更平滑、更连续的流动
    let pixel_offset = pixel_random * 0.3;
    
    // Wide blending window (8.0x) for ultra-smooth per-pixel transition
    // 宽混合窗口（8.0x）以实现超平滑的每像素过渡
    let pixel_progress = clamp((global_progress - pixel_offset) * 8.0, 0.0, 1.0);
    
    // Smoothstep for S-curve easing eliminates perceived stepping
    // Smoothstep 的 S 曲线缓动消除可感知的步进
    let smooth_progress = smoothstep(0.0, 1.0, pixel_progress);
    
    // Additional ease-out for gentle finish
    // 额外的 ease-out 以实现柔和结束
    let eased_progress = smooth_progress * (2.0 - smooth_progress);
    
    // Clean cutoff only at the very end
    // 仅在最末端干净截断
    let final_alpha = select(eased_progress, 1.0, in.color.r >= 0.88);
    
    return vec4<f32>(bw, bw, bw, tex_color.a * final_alpha);
}
