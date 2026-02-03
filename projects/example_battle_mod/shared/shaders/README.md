# 着色器目录说明 / Shader Directory

由于 Bevy 的 `Material2d` trait 限制，着色器路径在**编译时**确定，无法运行时配置。
因此，着色器文件必须存放在固定的目录结构中。

Due to Bevy's `Material2d` trait limitations, shader paths are determined at **compile time**
and cannot be configured at runtime. Therefore, shader files must be placed in fixed directory
structures.

---

## 必需文件 / Required Files

以下着色器文件必须存在于此目录：
The following shader files must exist in this directory:

| 文件名 / Filename | 用途 / Purpose |
|------------------|----------------|
| `hp_bar_sprite.wgsl` | HP 血条着色器 / HP bar shader |
| `pixel_outline.wgsl` | 像素描边着色器 / Pixel outline shader |
| `ui_solid_fill.wgsl` | UI 填充着色器 / UI solid fill shader |

---

## 自定义 / Customization

您可以修改这些文件的**内容**以自定义视觉效果，但**文件名和路径**必须保持不变。

You can modify the **contents** of these files to customize visual effects, but the
**filenames and paths** must remain unchanged.

---

## 技术原因 / Technical Reason

Bevy 的 `Material2d::fragment_shader()` 是静态方法，返回 `ShaderRef`。
该方法在编译时确定，无法在运行时从配置读取。

Bevy's `Material2d::fragment_shader()` is a static method that returns a `ShaderRef`.
This method is determined at compile time and cannot be read from configuration at runtime.

```rust
// 示例 / Example:
impl Material2d for HPBarMaterial {
    fn fragment_shader() -> ShaderRef {
        // 这里的路径是硬编码的 / The path here is hardcoded
        "shared/shaders/hp_bar_sprite.wgsl".into()
    }
}
```

---

## 注意事项 / Notes

- `shared/` 目录下的着色器在所有 Mod 之间共享
- `assets/shaders/` 目录下的着色器是 Mod 特定的
- 修改着色器后需要重新编译引擎才能生效
- 如果缺少必需的着色器文件，引擎可能会 panic

- Shaders under `shared/` directory are shared among all Mods
- Shaders under `assets/shaders/` directory are Mod-specific
- Engine must be recompiled after modifying shaders for changes to take effect
- Engine may panic if required shader files are missing
