// Black and White Tilemap Shader for bevy_ecs_tilemap
// bevy_ecs_tilemap 的黑白瓦片地图着色器
//
// Converts tilemap textures to black/gray/white with fade control and diffusion noise.
// in.color.r is used as fade value:
//   0.0 = pure black, 0.5 = normal B&W&Gray, 1.0 = pure white
// During transition, tile pixels "diffuse" from white with gray transition.
// 将瓦片地图纹理转换为黑/灰/白带淡入控制和扩散噪点。
// in.color.r 用作淡入值：
//   0.0 = 纯黑，0.5 = 正常黑白灰，1.0 = 纯白
// 过渡期间，瓦片像素从白色"扩散"并经过灰色过渡。

#import bevy_ecs_tilemap::common::{tilemap_data, sprite_texture, sprite_sampler}
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

// ========== BLACK/WHITE/GRAY THRESHOLDS ==========
// Thresholds for three-tone rendering: black, gray, white
// 三色调渲染的阈值：黑、灰、白
const THRESHOLD_DARK: f32 = 0.15;   // Below this = black / 低于此值 = 黑色
const THRESHOLD_LIGHT: f32 = 0.2;  // Above this = white / 高于此值 = 白色
                                      // Between = gray / 之间 = 灰色

// Simple hash function for pixel noise
// 用于像素噪点的简单哈希函数
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Hash function that returns vec2 for 2D randomness
// 返回 vec2 的哈希函数用于 2D 随机性
fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
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

    // Sample texture with subtle random offset for texture variation
    // 使用微妙的随机偏移采样纹理以产生纹理变化
    let tile_hash = hash22(vec2<f32>(f32(in.storage_position.x), f32(in.storage_position.y)));
    let texture_jitter = (tile_hash * 2.0 - 1.0) * 0.3 / tilemap_data.texture_size; // Small jitter in texture space
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset + texture_jitter);
    #else
    let tile_hash = hash22(vec2<f32>(f32(in.storage_position.x), f32(in.storage_position.y)));
    let texture_jitter = (tile_hash * 2.0 - 1.0) * 0.3 / tilemap_data.texture_size;
    let tex_color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + texture_jitter, in.tile_id);
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
    
    // Three-tone rendering: black, gray, white
    // 三色调渲染：黑、灰、白
    var tone: f32;
    if (luminance < THRESHOLD_DARK) {
        tone = 0.0; // Black / 黑色
    } else if (luminance < THRESHOLD_LIGHT) {
        tone = 0.5; // Gray / 灰色
    } else {
        tone = 1.0; // White / 白色
    }
    
    // Calculate tile pixel position (aligned to tile's pixel grid)
    // 计算瓦片像素位置（对齐到瓦片的像素网格）
    let tile_pixel = floor(in.uv.zw * tilemap_data.tile_size);
    
    // Combine with tile grid position for unique per-pixel value across the map
    // 与瓦片网格位置组合，得到整个地图中每个像素的唯一值
    let world_pixel = vec2<f32>(f32(in.storage_position.x), f32(in.storage_position.y)) * tilemap_data.tile_size + tile_pixel;
    
    // Generate diffusion-based noise
    // Base noise value for this pixel
    // 生成基于扩散的噪点
    // 此像素的基础噪点值
    let base_noise = hash21(world_pixel);
    
    // Distance from tile center for radial diffusion effect
    // 距离瓦片中心的距离用于径向扩散效果
    let tile_center = tilemap_data.tile_size * 0.5;
    let dist_from_center = length(tile_pixel - tile_center) / (tilemap_data.tile_size.x * 0.5);
    
    // Combine distance and noise for diffusion pattern
    // Inner pixels generate faster, outer pixels slower
    // 结合距离和噪点产生扩散模式
    // 内部像素生成更快，外部像素更慢
    let diffusion_factor = base_noise * (1.0 - dist_from_center * 0.3);
    
    // Apply fade with three-stage transition:
    // Stage 1 (fade 1.0 -> 0.833): white to gray
    // Stage 2 (fade 0.833 -> 0.667): gray to final tone
    // Stage 3 (fade 0.667 -> 0.5): fully revealed
    // 使用三阶段过渡应用淡入：
    // 阶段 1 (fade 1.0 -> 0.833)：白色到灰色
    // 阶段 2 (fade 0.833 -> 0.667)：灰色到最终色调
    // 阶段 3 (fade 0.667 -> 0.5)：完全显现
    var final_color: f32;
    if (fade > 0.833) {
        // Stage 1: White to gray transition
        // 阶段 1：白色到灰色过渡
        let stage_progress = (1.0 - fade) * 6.0; // 0 to 1 over this stage
        if (diffusion_factor < stage_progress) {
            final_color = 0.5; // Gray
        } else {
            final_color = 1.0; // White
        }
    } else if (fade > 0.667) {
        // Stage 2: Gray to tone transition
        // 阶段 2：灰色到色调过渡
        let stage_progress = (0.833 - fade) * 6.0; // 0 to 1 over this stage
        if (diffusion_factor < stage_progress) {
            final_color = tone; // Final tone
        } else {
            final_color = 0.5; // Gray
        }
    } else if (fade > 0.5) {
        // Stage 3: Final refinement
        // 阶段 3：最终细化
        let stage_progress = (0.667 - fade) * 6.0;
        if (diffusion_factor < stage_progress) {
            final_color = tone;
        } else {
            final_color = tone; // Mostly done, slight variation
        }
    } else {
        // Fully revealed
        // 完全显现
        final_color = tone;
    }
    
    // ========== PIXEL GENERATION ANIMATION ==========
    // Smooth, continuous alpha-based animation without any stepping
    // 完全连续的基于alpha的动画，无任何步进
    
    // When fade = 1.0 (initial state), tile should be fully visible and white
    // The animation progresses as fade decreases from 1.0 to 0.5
    // 当 fade = 1.0（初始状态）时，瓦片应完全可见且为白色
    // 动画随着 fade 从 1.0 减少到 0.5 而进行
    
    // Calculate overall animation progress (0 = start/white, 1 = complete/normal)
    // 计算整体动画进度（0 = 开始/白色，1 = 完成/正常）
    let global_progress = clamp((1.0 - fade) * 2.0, 0.0, 1.0);
    
    // Each pixel has staggered timing for wave effect
    // Use smaller offset range (0-0.3) for smoother overall flow
    // 每个像素有交错的时机以产生波浪效果
    // 使用更小的偏移范围（0-0.3）以实现更平滑的整体流动
    let pixel_offset = diffusion_factor * 0.3;
    
    // Calculate this pixel's individual progress with smooth blending window
    // Wider window (8.0x multiplier) = smoother, more gradual transition
    // 计算此像素的个体进度，使用平滑混合窗口
    // 更宽的窗口（8.0x 乘数）= 更平滑、更渐进的过渡
    let pixel_progress = clamp((global_progress - pixel_offset) * 8.0, 0.0, 1.0);
    
    // Apply smoothstep for ultra-smooth S-curve easing
    // This eliminates any perception of "steps" or "jumps"
    // 应用 smoothstep 以实现超平滑的 S 曲线缓动
    // 这消除了任何"步进"或"跳跃"的感知
    let smooth_progress = smoothstep(0.0, 1.0, pixel_progress);
    
    // Apply additional ease-out for even gentler finish
    // f(t) = t * (2 - t)
    // 应用额外的 ease-out 以实现更柔和的结束
    let eased_progress = smooth_progress * (2.0 - smooth_progress);
    
    // Alpha is always 1.0 - tiles are always fully visible
    // The "animation" is in the color transition, not alpha
    // Alpha 始终为 1.0 - 瓦片始终完全可见
    // "动画"在于颜色过渡，而不是 alpha
    let pixel_alpha = 1.0;
    
    return vec4<f32>(final_color, final_color, final_color, tex_color.a * pixel_alpha);
}
