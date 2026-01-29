//! # shaders.rs
//!
//! # shaders.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module loads WGSL shader code from external files for use with SdfMaterial.
//!
//! 本模块从外部文件加载 WGSL 着色器代码供 SdfMaterial 使用。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! Shaders are stored as separate .wgsl files for easier modification.
//!
//! 着色器存储为独立的 .wgsl 文件以便于修改。

/// Load UI solid fill shader body from external file.
///
/// 从外部文件加载 UI 实体填充着色器主体。
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

/// Load custom shader body from project directory for data-driven shader loading.
/// The path is relative to the project's root directory (projects/MOD_NAME/).
///
/// 从项目目录加载自定义着色器主体，用于数据驱动的着色器加载。
/// 路径相对于项目根目录 (projects/MOD_NAME/)。
///
/// # Arguments
/// * `path` - Relative path to the shader file, e.g., "shared/shaders/hp_bar.wgsl"
///
/// # Returns
/// The shader source code as a String, or a fallback magenta shader on error.
pub fn load_custom_shader_body(path: &str) -> String {
    let config = crate::config::load_config();
    let full_path = format!("projects/{}/{}", config.project.mod_name, path);

    std::fs::read_to_string(&full_path).unwrap_or_else(|e| {
        eprintln!(
            "Warning: Failed to load custom shader from {}: {}",
            full_path, e
        );
        // Return magenta color to indicate shader loading error
        // 返回品红色以指示着色器加载错误
        "return vec4<f32>(1.0, 0.0, 1.0, 1.0);".to_string()
    })
}
