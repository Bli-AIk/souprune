// Pixel Outline Sprite Shader - Red 1-pixel outline effect for chase state
// 像素描边精灵着色器 - 追逐战状态的红色1像素描边效果
//
// This shader creates an outline by sampling adjacent pixels within a sprite atlas rect.
// The outline is drawn OUTSIDE the sprite bounds (on transparent pixels adjacent to opaque ones).
// 此着色器通过在图集精灵区域内采样相邻像素创建描边。
// 描边绘制在精灵边界外部（在与不透明像素相邻的透明像素上）。
//
// The mesh is 2 pixels larger than the sprite (1 pixel border on each side).
// UV coordinates 0-1 map to the entire mesh, so we need to:
// 1. Map mesh UV to sprite UV (accounting for the 1px border)
// 2. For border pixels, check if adjacent sprite pixels are opaque
//
// Uniform data:
// - params: (r, g, b, a) = outline color RGB and alpha
// - uv_rect: (min_u, min_v, max_u, max_v) = UV coordinates of sprite in atlas
// - flip: (flip_x, flip_y, 0, 0) = flip flags (0.0 or 1.0)

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> params: vec4<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var<uniform> uv_rect: vec4<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<uniform> flip: vec4<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(3)
var base_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(4)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let outline_color = vec3<f32>(params.r, params.g, params.b);
    let outline_alpha = params.a;
    
    // UV rect bounds (min_u, min_v, max_u, max_v) - this is the sprite's UV in the atlas
    let uv_min = uv_rect.xy;
    let uv_max = uv_rect.zw;
    let uv_size = uv_max - uv_min;
    
    // Get texture dimensions for pixel-perfect calculations
    let tex_size = vec2<f32>(textureDimensions(base_texture));
    let pixel_size = 1.0 / tex_size;
    
    // The sprite size in UV space
    let sprite_uv_size = uv_size;
    
    // The mesh is sprite_size + 2 pixels, so mesh UV needs to be adjusted
    // to account for the 1 pixel border on each side
    // Border in UV space = 1 pixel / sprite_pixels = pixel_size / uv_size (approximately)
    // Actually, we need: mesh_size = sprite_size + 2, so:
    // sprite_uv maps to center portion of mesh_uv
    // mesh_uv 0 -> sprite_uv -1px, mesh_uv 1 -> sprite_uv +1px
    
    // Calculate the border size in mesh UV space
    // If sprite is W pixels and mesh is W+2 pixels, then:
    // border_ratio = 1 / (W + 2) in mesh UV space
    // But we need to work in atlas UV space
    
    // Sprite size in pixels (derived from UV rect)
    let sprite_pixels = uv_size * tex_size;
    let mesh_pixels = sprite_pixels + vec2<f32>(2.0, 2.0);
    
    // Border size as fraction of mesh
    let border_ratio = vec2<f32>(1.0, 1.0) / mesh_pixels;
    
    // Apply flip to mesh UV
    var mesh_uv = in.uv;
    if (flip.x > 0.5) {
        mesh_uv.x = 1.0 - mesh_uv.x;
    }
    if (flip.y > 0.5) {
        mesh_uv.y = 1.0 - mesh_uv.y;
    }
    
    // Map mesh UV to sprite UV
    // mesh_uv in [0, border_ratio] -> outside sprite (left/bottom border)
    // mesh_uv in [border_ratio, 1-border_ratio] -> inside sprite
    // mesh_uv in [1-border_ratio, 1] -> outside sprite (right/top border)
    
    // Remap mesh UV to sprite UV: subtract border, scale to sprite range
    let sprite_uv_normalized = (mesh_uv - border_ratio) / (vec2<f32>(1.0, 1.0) - 2.0 * border_ratio);
    
    // Convert to atlas UV
    let atlas_uv = uv_min + sprite_uv_normalized * uv_size;
    
    // Check if we're inside the sprite bounds (in normalized sprite UV)
    let inside_sprite = sprite_uv_normalized.x >= 0.0 && sprite_uv_normalized.x <= 1.0 &&
                        sprite_uv_normalized.y >= 0.0 && sprite_uv_normalized.y <= 1.0;
    
    // Early exit if no outline needed
    if (outline_alpha <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    // If inside sprite bounds, check if this is a transparent pixel adjacent to opaque pixel
    // Only draw outline on transparent pixels that are next to opaque pixels (outer edge only)
    // 如果在精灵边界内，检查这是否是与不透明像素相邻的透明像素
    // 只在与不透明像素相邻的透明像素上绘制描边（仅外边缘）
    if (inside_sprite) {
        let clamped_uv = clamp(atlas_uv, uv_min, uv_max - pixel_size * 0.5);
        let current_pixel = textureSample(base_texture, base_sampler, clamped_uv);
        
        // If current pixel is opaque, don't draw anything (we don't draw on top of sprite)
        // 如果当前像素是不透明的，不绘制任何东西（我们不在精灵上方绘制）
        if (current_pixel.a > 0.1) {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        
        // Current pixel is transparent, check if any adjacent pixel is opaque
        // 当前像素是透明的，检查是否有相邻像素是不透明的
        let up_uv = clamp(atlas_uv + vec2<f32>(0.0, pixel_size.y), uv_min, uv_max);
        let down_uv = clamp(atlas_uv - vec2<f32>(0.0, pixel_size.y), uv_min, uv_max);
        let left_uv = clamp(atlas_uv - vec2<f32>(pixel_size.x, 0.0), uv_min, uv_max);
        let right_uv = clamp(atlas_uv + vec2<f32>(pixel_size.x, 0.0), uv_min, uv_max);
        
        let up = textureSample(base_texture, base_sampler, up_uv);
        let down = textureSample(base_texture, base_sampler, down_uv);
        let left = textureSample(base_texture, base_sampler, left_uv);
        let right = textureSample(base_texture, base_sampler, right_uv);
        
        let max_adjacent_alpha = max(max(up.a, down.a), max(left.a, right.a));
        if (max_adjacent_alpha > 0.1) {
            // This transparent pixel is adjacent to an opaque pixel - draw outline
            return vec4<f32>(outline_color, outline_alpha);
        }
        
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
    // Outside sprite bounds - this is the 1px border area
    // Check if the nearest sprite edge pixel is opaque to draw outline
    // 在精灵边界外 - 这是1像素边框区域
    // 检查最近的精灵边缘像素是否不透明以绘制描边
    let clamped_sprite_uv = clamp(sprite_uv_normalized, vec2<f32>(0.0), vec2<f32>(1.0));
    let edge_atlas_uv = uv_min + clamped_sprite_uv * uv_size;
    let edge_pixel = textureSample(base_texture, base_sampler, clamp(edge_atlas_uv, uv_min, uv_max - pixel_size * 0.5));
    
    // If the nearest edge pixel is opaque, draw outline
    if (edge_pixel.a > 0.1) {
        return vec4<f32>(outline_color, outline_alpha);
    }
    
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
