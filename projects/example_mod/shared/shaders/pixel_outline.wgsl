// Pixel Outline Sprite Shader - Red 1-pixel outline effect for chase state
// 像素描边精灵着色器 - 追逐战状态的红色1像素描边效果
//
// This shader creates an outline by sampling adjacent pixels within a sprite atlas rect.
// The outline is drawn OUTSIDE the sprite bounds (on transparent pixels adjacent to opaque ones).
// 此着色器通过在图集精灵区域内采样相邻像素创建描边。
// 描边绘制在精灵边界外部（在与不透明像素相邻的透明像素上）。
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
    
    // UV rect bounds (min_u, min_v, max_u, max_v)
    let uv_min = uv_rect.xy;
    let uv_max = uv_rect.zw;
    let uv_size = uv_max - uv_min;
    
    // Apply flip to mesh UV
    var mesh_uv = in.uv;
    if (flip.x > 0.5) {
        mesh_uv.x = 1.0 - mesh_uv.x;
    }
    if (flip.y > 0.5) {
        mesh_uv.y = 1.0 - mesh_uv.y;
    }
    
    // Convert mesh UV (0-1) to atlas UV
    let atlas_uv = uv_min + mesh_uv * uv_size;
    
    // Get texture dimensions for pixel-perfect offset
    let tex_size = vec2<f32>(textureDimensions(base_texture));
    let pixel_size = 1.0 / tex_size;
    
    // Sample the original pixel (clamped to UV rect)
    let clamped_uv = clamp(atlas_uv, uv_min, uv_max - pixel_size * 0.5);
    let original = textureSample(base_texture, base_sampler, clamped_uv);
    
    // If outline is disabled, return original
    if (outline_alpha <= 0.0) {
        return original;
    }
    
    // If this pixel is opaque, return original color (sprite pixels are rendered normally)
    if (original.a > 0.1) {
        return original;
    }
    
    // For transparent pixels, check if any adjacent pixel (within the atlas rect) is opaque
    // This allows outline to be drawn on the edge of the sprite
    
    // Sample adjacent pixels - allow sampling slightly outside UV rect for edge detection
    // but clamp to valid texture coordinates
    let up_uv = clamp(atlas_uv + vec2<f32>(0.0, pixel_size.y), uv_min, uv_max);
    let down_uv = clamp(atlas_uv - vec2<f32>(0.0, pixel_size.y), uv_min, uv_max);
    let left_uv = clamp(atlas_uv - vec2<f32>(pixel_size.x, 0.0), uv_min, uv_max);
    let right_uv = clamp(atlas_uv + vec2<f32>(pixel_size.x, 0.0), uv_min, uv_max);
    
    let up = textureSample(base_texture, base_sampler, up_uv);
    let down = textureSample(base_texture, base_sampler, down_uv);
    let left = textureSample(base_texture, base_sampler, left_uv);
    let right = textureSample(base_texture, base_sampler, right_uv);
    
    // Check if any adjacent pixel is opaque
    let adjacent_alpha = max(max(up.a, down.a), max(left.a, right.a));
    
    // If any adjacent pixel is opaque, this is an outline pixel
    if (adjacent_alpha > 0.1) {
        return vec4<f32>(outline_color, outline_alpha);
    }
    
    // Otherwise, return transparent
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
