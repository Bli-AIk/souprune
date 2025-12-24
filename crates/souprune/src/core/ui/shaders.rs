//! # shaders.rs
//!
//! # shaders.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module loads WGSL shader code from external files for use with bevy_smud.
//!
//! 本模块从外部文件加载 WGSL 着色器代码供 bevy_smud 使用。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! Shaders are stored as separate .wgsl files for easier modification.
//!
//! 着色器存储为独立的 .wgsl 文件以便于修改。

/// Load UI solid fill shader body from external file.
pub fn load_ui_solid_fill_body() -> String {
    let config = crate::config::load_config();
    let shader_path = format!(
        "projects/{}/shared/shaders/ui_solid_fill.wgsl",
        config.project.mod_name
    );

    std::fs::read_to_string(&shader_path).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load shader from {}: {}", shader_path, e);
        "let a = select(0.0, 1.0, input.distance <= 0.0);\nreturn vec4<f32>(input.color.rgb, a);"
            .to_string()
    })
}
