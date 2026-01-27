// Gaussian Blur Horizontal Pass
// 
// This shader implements the horizontal pass of a separable Gaussian blur.
// Uses optimized 1D kernel with proper transparent boundary handling.
//
// Key fix: Out-of-bounds samples contribute zero color but WEIGHT is always accumulated.
// This ensures edge pixels fade to transparent instead of stretching (Clamp).
//
// The separable approach splits 2D Gaussian blur into two 1D passes:
// - First: Horizontal blur (this shader)
// - Second: Vertical blur (gaussian_blur_v.wgsl)
// 
// This reduces complexity from O(radius²) to O(2 * radius).

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> blur_params: vec4<f32>;  // x = radius, y = tex_width, z = tex_height, w = unused
@group(2) @binding(1) var base_texture: texture_2d<f32>;
@group(2) @binding(2) var base_sampler: sampler;

fn gaussian_weight(offset: f32, sigma: f32) -> f32 {
    return exp(-(offset * offset) / (2.0 * sigma * sigma));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let radius = blur_params.x;
    let tex_width = blur_params.y;
    
    // Early exit for no blur
    if radius < 0.5 {
        return textureSample(base_texture, base_sampler, mesh.uv);
    }
    
    // Sigma = radius / 2.0 for softer, more natural light diffusion (closer to Alight Motion)
    let sigma = max(radius / 2.0, 0.01);
    let pixel_size = 1.0 / tex_width;
    
    var total_color = vec4<f32>(0.0);
    var total_weight = 0.0;
    
    // Cover 3*sigma range with hard cap for performance
    // For separable blur, 60-100 samples per axis is acceptable (O(N) vs O(N²))
    let num_samples = i32(min(ceil(sigma * 3.0), 100.0));
    
    // 1. Center sample
    total_color += textureSample(base_texture, base_sampler, mesh.uv);
    total_weight += 1.0;
    
    // 2. Symmetric sampling (left and right)
    for (var i = 1; i <= num_samples; i = i + 1) {
        let offset = f32(i);
        let weight = gaussian_weight(offset, sigma);
        
        // Performance: truncate negligible weights
        if weight < 0.001 {
            break;
        }
        
        let offset_uv = offset * pixel_size;
        
        // --- KEY FIX: Transparent boundary logic ---
        
        // Right sample
        let uv_right = mesh.uv + vec2<f32>(offset_uv, 0.0);
        if uv_right.x <= 1.0 && uv_right.x >= 0.0 {
            total_color += textureSample(base_texture, base_sampler, uv_right) * weight;
        }
        // Out-of-bounds: don't add color (equivalent to adding vec4(0.0) * weight)
        
        // Left sample  
        let uv_left = mesh.uv - vec2<f32>(offset_uv, 0.0);
        if uv_left.x >= 0.0 && uv_left.x <= 1.0 {
            total_color += textureSample(base_texture, base_sampler, uv_left) * weight;
        }
        
        // CRITICAL: Always accumulate weight regardless of bounds!
        // This ensures edge regions get larger denominator -> color trends to transparent
        total_weight += 2.0 * weight;
    }
    
    // Normalize
    if total_weight > 0.0001 {
        return total_color / total_weight;
    }
    return vec4<f32>(0.0);
}
