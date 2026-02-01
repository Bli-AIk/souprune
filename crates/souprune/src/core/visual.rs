//! # visual.rs
//!
//! # 视觉描述符模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Provides a unified visual descriptor system for sprites and animations.
//! Supports shorthand path references that are automatically resolved.
//!
//! 提供统一的视觉描述符系统，用于精灵和动画。
//! 支持自动解析的简写路径引用。
//!
//! ## Path Resolution
//!
//! ## 路径解析
//!
//! - Simple name: `"heart"` → searches textures directory recursively
//! - Relative path: `"battle/bullets/spear"` → relative to textures directory
//! - File extension optional: automatically detected
//!
//! - 简单名称：`"heart"` → 在纹理目录中递归搜索
//! - 相对路径：`"battle/bullets/spear"` → 相对于纹理目录
//! - 文件扩展名可选：自动检测

use crate::config::{ResourcePaths, load_config};
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Unified visual reference.
///
/// 统一的视觉引用。
///
/// ## Examples
///
/// ```ron
/// // Simple path (most common)
/// visual: "heart"
///
/// // With extra configuration
/// visual: (
///     visual: "heart",
///     color: Rgba(r: 1.0, g: 0.0, b: 0.0, a: 1.0),
///     flip_x: true,
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[serde(untagged)]
pub enum Visual {
    /// Path reference (recommended for most cases).
    ///
    /// 路径引用（大多数情况下推荐）。
    Path(String),

    /// Full configuration with additional properties.
    ///
    /// 带有额外属性的完整配置。
    Config(VisualConfig),
}

impl Default for Visual {
    fn default() -> Self {
        Self::Path(String::new())
    }
}

impl Visual {
    /// Get the visual path string.
    ///
    /// 获取视觉路径字符串。
    pub fn path(&self) -> &str {
        match self {
            Visual::Path(p) => p,
            Visual::Config(c) => &c.visual,
        }
    }

    /// Get color override if specified.
    ///
    /// 获取颜色覆盖（如果指定）。
    pub fn color(&self) -> Option<Color> {
        match self {
            Visual::Path(_) => None,
            Visual::Config(c) => c.color,
        }
    }

    /// Get flip_x setting.
    ///
    /// 获取 flip_x 设置。
    pub fn flip_x(&self) -> bool {
        match self {
            Visual::Path(_) => false,
            Visual::Config(c) => c.flip_x,
        }
    }

    /// Get flip_y setting.
    ///
    /// 获取 flip_y 设置。
    pub fn flip_y(&self) -> bool {
        match self {
            Visual::Path(_) => false,
            Visual::Config(c) => c.flip_y,
        }
    }

    /// Get pivot override if specified.
    ///
    /// 获取锚点覆盖（如果指定）。
    pub fn pivot(&self) -> Option<Vec2> {
        match self {
            Visual::Path(_) => None,
            Visual::Config(c) => c.pivot,
        }
    }

    /// Get frame duration override for animations.
    ///
    /// 获取动画的帧持续时间覆盖。
    pub fn frame_duration(&self) -> Option<f32> {
        match self {
            Visual::Path(_) => None,
            Visual::Config(c) => c.frame_duration,
        }
    }
}

/// Visual configuration with additional properties.
///
/// 带有额外属性的视觉配置。
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct VisualConfig {
    /// Path to the visual resource.
    ///
    /// 视觉资源的路径。
    pub visual: String,

    /// Color tint override.
    ///
    /// 颜色叠加覆盖。
    #[serde(default)]
    pub color: Option<Color>,

    /// Horizontal flip.
    ///
    /// 水平翻转。
    #[serde(default)]
    pub flip_x: bool,

    /// Vertical flip.
    ///
    /// 垂直翻转。
    #[serde(default)]
    pub flip_y: bool,

    /// Pivot point override.
    ///
    /// 锚点覆盖。
    #[serde(default)]
    pub pivot: Option<Vec2>,

    /// Frame duration override for animations (seconds).
    ///
    /// 动画的帧持续时间覆盖（秒）。
    #[serde(default)]
    pub frame_duration: Option<f32>,
}

/// Result of resolving a visual path.
///
/// 解析视觉路径的结果。
#[derive(Debug, Clone)]
pub enum ResolvedVisual {
    /// Static sprite image.
    ///
    /// 静态精灵图片。
    Sprite(PathBuf),

    /// Frame animation (directory containing numbered images).
    ///
    /// 帧动画（包含编号图片的目录）。
    FrameAnimation(PathBuf),

    /// Character animation state machine (*.character.ron).
    ///
    /// 角色动画状态机（*.character.ron）。
    CharacterAnimation(PathBuf),
}

/// Default frame duration for animations (20 FPS).
///
/// 动画的默认帧持续时间（20 FPS）。
pub const DEFAULT_FRAME_DURATION: f32 = 0.05;

/// Supported image extensions for automatic detection.
///
/// 自动检测支持的图片扩展名。
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

/// Resolve a visual path to its full location and type.
///
/// 将视觉路径解析为完整位置和类型。
///
/// ## Resolution Order
///
/// ## 解析顺序
///
/// 1. If path ends with `.character.ron` → CharacterAnimation
/// 2. If path points to a directory → FrameAnimation
/// 3. If path points to a file → Sprite
/// 4. If no extension, search for matching file/directory
///
/// 1. 如果路径以 `.character.ron` 结尾 → CharacterAnimation
/// 2. 如果路径指向目录 → FrameAnimation
/// 3. 如果路径指向文件 → Sprite
/// 4. 如果没有扩展名，搜索匹配的文件/目录
pub fn resolve_visual_path(input: &str, mod_name: &str) -> Option<ResolvedVisual> {
    let config = load_config();
    resolve_visual_path_with_resources(input, mod_name, &config.resources)
}

/// Resolve a visual path using the provided resource paths configuration.
///
/// 使用提供的资源路径配置解析视觉路径。
pub fn resolve_visual_path_with_resources(
    input: &str,
    mod_name: &str,
    resources: &ResourcePaths,
) -> Option<ResolvedVisual> {
    // Character animation files are identified by extension
    if input.ends_with(".character.ron") {
        let full_path = build_texture_path(mod_name, resources, input);
        if full_path.exists() {
            return Some(ResolvedVisual::CharacterAnimation(full_path));
        }
        // Also search recursively
        if let Some(found) = search_texture_recursive(mod_name, resources, input) {
            return Some(ResolvedVisual::CharacterAnimation(found));
        }
        return None;
    }

    // Check if input has a file extension
    let has_extension = Path::new(input)
        .extension()
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_str().unwrap_or("")))
        .unwrap_or(false);

    if has_extension {
        // Direct file path with extension
        let full_path = build_texture_path(mod_name, resources, input);
        if full_path.exists() {
            return Some(ResolvedVisual::Sprite(full_path));
        }
    }

    // Try to find as a directory (frame animation)
    let dir_path = build_texture_path(mod_name, resources, input);
    if dir_path.is_dir() {
        return Some(ResolvedVisual::FrameAnimation(dir_path));
    }

    // Try adding common extensions
    for ext in IMAGE_EXTENSIONS {
        let with_ext = format!("{}.{}", input, ext);
        let full_path = build_texture_path(mod_name, resources, &with_ext);
        if full_path.exists() {
            return Some(ResolvedVisual::Sprite(full_path));
        }
    }

    // If simple name (no path separator), search recursively
    if !input.contains('/') && !input.contains('\\') {
        if let Some(found) = search_texture_recursive(mod_name, resources, input) {
            if found.is_dir() {
                return Some(ResolvedVisual::FrameAnimation(found));
            } else {
                return Some(ResolvedVisual::Sprite(found));
            }
        }
    }

    None
}

/// Build the full texture path from mod name and relative path.
///
/// 从 mod 名称和相对路径构建完整的纹理路径。
fn build_texture_path(mod_name: &str, resources: &ResourcePaths, relative: &str) -> PathBuf {
    Path::new("projects")
        .join(mod_name)
        .join(&resources.textures)
        .join(relative)
}

/// Search for a file or directory by name recursively in the textures directory.
/// Prioritizes files over directories to avoid mistaking a single-image directory as frame animation.
///
/// 在纹理目录中按名称递归搜索文件或目录。
/// 优先返回文件而非目录，以避免将单图片目录误判为帧动画。
fn search_texture_recursive(
    mod_name: &str,
    resources: &ResourcePaths,
    name: &str,
) -> Option<PathBuf> {
    let textures_root = Path::new("projects")
        .join(mod_name)
        .join(&resources.textures);

    if !textures_root.exists() {
        return None;
    }

    let mut file_matches = Vec::new();
    let mut dir_matches = Vec::new();
    search_recursive_inner(&textures_root, name, &mut file_matches, &mut dir_matches);

    // Prioritize file matches over directory matches
    // This avoids mistaking "spear/" directory with "spear.png" inside as frame animation
    if !file_matches.is_empty() {
        if file_matches.len() > 1 {
            warn!(
                "Multiple file matches found for '{}': {:?}. Using first match.",
                name, file_matches
            );
        }
        return file_matches.into_iter().next();
    }

    // Only use directory match if it's a valid frame animation directory
    // (contains multiple numbered images like 0.png, 1.png, etc.)
    for dir in dir_matches {
        if is_frame_animation_directory(&dir) {
            return Some(dir);
        }
    }

    None
}

/// Check if a directory looks like a frame animation directory.
/// Frame animation directories should contain multiple images.
///
/// 检查目录是否看起来像帧动画目录。
/// 帧动画目录应包含多个图片。
fn is_frame_animation_directory(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };

    let mut image_count = 0;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if IMAGE_EXTENSIONS.contains(&ext) {
                    image_count += 1;
                }
            }
        }
    }

    // A valid frame animation has multiple images (2 or more)
    image_count >= 2
}

/// Inner recursive search function.
/// Separates file matches and directory matches.
///
/// 内部递归搜索函数。
/// 将文件匹配和目录匹配分开。
fn search_recursive_inner(
    dir: &Path,
    name: &str,
    file_results: &mut Vec<PathBuf>,
    dir_results: &mut Vec<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

        // Check for exact match (with or without extension)
        if stem == name {
            if path.is_dir() {
                dir_results.push(path.clone());
            } else {
                file_results.push(path.clone());
            }
        }

        // Recurse into directories
        if path.is_dir() && !file_name.starts_with('.') {
            search_recursive_inner(&path, name, file_results, dir_results);
        }
    }
}

/// Get the relative path for asset loading from a resolved visual.
///
/// 从解析的视觉获取用于资源加载的相对路径。
pub fn get_asset_path(resolved: &ResolvedVisual, mod_name: &str) -> String {
    let path = match resolved {
        ResolvedVisual::Sprite(p) => p,
        ResolvedVisual::FrameAnimation(p) => p,
        ResolvedVisual::CharacterAnimation(p) => p,
    };

    // Strip "projects/{mod_name}/" prefix for asset server
    let prefix = format!("projects/{}/", mod_name);
    path.to_string_lossy()
        .strip_prefix(&prefix)
        .unwrap_or(&path.to_string_lossy())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_path_extraction() {
        let v1 = Visual::Path("heart".to_string());
        assert_eq!(v1.path(), "heart");
        assert!(!v1.flip_x());

        let v2 = Visual::Config(VisualConfig {
            visual: "spear".to_string(),
            color: Some(Color::srgba(1.0, 0.0, 0.0, 1.0)),
            flip_x: true,
            flip_y: false,
            pivot: None,
            frame_duration: Some(0.1),
        });
        assert_eq!(v2.path(), "spear");
        assert!(v2.flip_x());
        assert_eq!(v2.frame_duration(), Some(0.1));
    }

    #[test]
    fn test_visual_deserialize_path() {
        let ron_str = r#""heart""#;
        let v: Visual = ron::from_str(ron_str).unwrap();
        assert_eq!(v.path(), "heart");
    }

    #[test]
    fn test_visual_deserialize_config() {
        let ron_str = r#"(visual: "heart", flip_x: true)"#;
        let v: Visual = ron::from_str(ron_str).unwrap();
        assert_eq!(v.path(), "heart");
        assert!(v.flip_x());
    }
}
