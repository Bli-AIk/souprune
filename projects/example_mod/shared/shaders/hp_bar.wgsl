// Undertale-style HP Bar Shader with delayed white bar effect
// 带有延迟白条效果的 Undertale 风格血条着色器
//
// Data passed through input.color (vec4):
// 通过 input.color (vec4) 传递的数据：
//   - input.color.r: Current HP percentage (0.0 - 1.0, yellow bar)
//                    当前血量百分比（0.0 - 1.0，黄条）
//   - input.color.g: Delayed HP percentage (0.0 - 1.0, white bar)
//                    延迟血量百分比（0.0 - 1.0，白条）
//   - input.color.b: Box width in pixels (for coordinate calculation)
//                    盒子宽度（像素，用于坐标计算）
//   - input.color.a: Box height in pixels (for coordinate calculation)
//                    盒子高度（像素，用于坐标计算）
//
// Note: This shader uses bevy_smud's input structure where:
// 注意：此着色器使用 bevy_smud 的输入结构，其中：
//   - input.p: vec2<f32> - current pixel position relative to box center (SDF space)
//              当前像素相对于盒子中心的位置（SDF 空间）
//   - input.distance: f32 - SDF distance value (negative inside, positive outside)
//                     SDF 距离值（内部为负，外部为正）

let hp_ratio = input.color.r;
let lag_ratio = input.color.g;
let width = input.color.b;
let height = input.color.a;

// Map x coordinate from [-width/2, width/2] to [0, 1]
// 将 x 坐标从 [-width/2, width/2] 映射到 [0, 1]
let half_width = width / 2.0;
let t = (input.p.x + half_width) / width;

// Color definitions (Undertale style)
// 颜色定义（Undertale 风格）
let col_bg = vec4<f32>(1.0, 0.0, 0.0, 1.0);     // Red background (Max HP) / 红色背景（最大血量）
let col_lag = vec4<f32>(1.0, 1.0, 1.0, 1.0);    // White delayed bar (Loss) / 白色延迟条（损失）
let col_hp = vec4<f32>(1.0, 1.0, 0.0, 1.0);     // Yellow current bar (Current HP) / 黄色当前条（当前血量）

// Apply SDF masking - only draw inside the box
// 应用 SDF 遮罩 - 仅在盒子内部绘制
let inside_mask = select(0.0, 1.0, input.distance <= 0.0);

// Layer logic: yellow (current) > white (delayed) > red (background)
// 层级逻辑：黄色（当前）> 白色（延迟）> 红色（背景）
var final_color: vec4<f32>;
if (t < hp_ratio) {
    final_color = col_hp;
} else if (t < lag_ratio) {
    final_color = col_lag;
} else {
    final_color = col_bg;
}

return vec4<f32>(final_color.rgb, final_color.a * inside_mask);
