# 着色器目录说明 / Shader Directory

## 新特性：运行时动态着色器 / New Feature: Runtime Dynamic Shaders

从新版本开始，SoupRune 支持**运行时动态着色器**！

Starting from the new version, SoupRune supports **runtime dynamic shaders**!

### 工作原理 / How it works

通过 `DynamicMaterial2d` 系统和 `SpecializedMeshPipeline`，您现在可以：

Through the `DynamicMaterial2d` system and `SpecializedMeshPipeline`, you can now:

- ✅ 在 RON 配置文件中指定任意着色器路径
- ✅ 创建自定义着色器并放置在 Mod 目录中
- ✅ 无需重新编译引擎即可使用新着色器
- ✅ 热重载支持（修改着色器文件后自动重新加载）

- ✅ Specify arbitrary shader paths in RON config files
- ✅ Create custom shaders and place them in Mod directories
- ✅ Use new shaders without recompiling the engine
- ✅ Hot reload support (auto-reload after modifying shader files)

---

## 配置方式 / Configuration

在 `view_layout.ron` 文件中使用新的 `material` 字段：

Use the new `material` field in `view_layout.ron` files:

```ron
sprite: (
    visual: "procedural://white_pixel",
    material: (
        // 可以是任意路径 / Can be any path
        shader: "shared/shaders/hp_bar_sprite.wgsl",
        // 或者使用 mod:// 协议引用 Mod 目录中的着色器
        // Or use mod:// protocol to reference shaders in Mod directory
        // shader: "mod://my_awesome_mod/shaders/custom_effect.wgsl",
        params: {
            "hp_ratio": Expr("$player_hp / $player_hp_max"),
            "lag_ratio": Static(1.0),
            "half_width": Expr("40.0"),
            "alpha": Static(1.0),
        },
        animations: (
            lag: (
                source: "hp_ratio",
                target: "lag_ratio",
                delay: 0.2,
                duration: 0.4,
                easing: OutCirc,
            ),
        ),
    ),
)
```

---

## 内置着色器 / Built-in Shaders

以下着色器作为参考实现提供：
The following shaders are provided as reference implementations:

| 文件名 / Filename | 用途 / Purpose |
|------------------|----------------|
| `hp_bar_sprite.wgsl` | 玩家 HP 条 / Player HP bar |
| `enemy_hp_bar_sprite.wgsl` | 敌人 HP 条 / Enemy HP bar |
| `pixel_outline.wgsl` | 像素描边效果 / Pixel outline effect |
| `ui_solid_fill.wgsl` | UI 填充 / UI solid fill |

---

## 自定义着色器 / Custom Shaders

您可以创建自己的着色器！着色器必须：

You can create your own shaders! Shaders must:

1. 接受 `params` (Vec4) 和 `extra_params` (Vec4) uniform
2. 可选：使用纹理采样器

1. Accept `params` (Vec4) and `extra_params` (Vec4) uniforms
2. Optional: Use texture samplers

### 示例着色器模板 / Example Shader Template

```wgsl
#import bevy_sprite_render::mesh2d_view_bindings::view
#import bevy_sprite_render::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> params: vec4<f32>;
@group(2) @binding(1) var<uniform> extra_params: vec4<f32>;
@group(2) @binding(2) var base_texture: texture_2d<f32>;
@group(2) @binding(3) var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let my_param = params.x;
    
    // 您的自定义效果代码 / Your custom effect code here
    
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
```

---

## 注意事项 / Notes

- `shared/` 目录下的着色器在所有 Mod 之间共享
- Mod 可以通过 `mod://mod_name/path` 协议使用自己的着色器
- 着色器支持热重载，无需重启游戏

- Shaders under `shared/` directory are shared among all Mods
- Mods can use their own shaders via `mod://mod_name/path` protocol
- Shaders support hot reload, no game restart required
