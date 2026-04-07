//! # visual.rs
//!
//! # 视觉资源定位模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Provides a unified visual resource locator for sprites and animations.
//! Supports shorthand path references that are automatically resolved and type-detected.
//!
//! 提供统一的视觉资源定位器，用于精灵和动画。
//! 支持自动解析和类型检测的简写路径引用。
//!
//! ## Design Philosophy
//!
//! ## 设计哲学
//!
//! Visual is a "source file locator" - it only handles finding and classifying resources.
//! Rendering properties (color, flip, shader, etc.) belong to the consuming systems.
//!
//! Visual 是一个"源文件定位器" - 它只处理资源的查找和分类。
//! 渲染属性（颜色、翻转、着色器等）属于使用它的系统。
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

use crate::config::load_config;
use crate::core::resource_resolver;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Unified visual resource path reference.
///
/// 统一的视觉资源路径引用。
///
/// This is a simple wrapper around a path string that supports:
/// - Automatic type detection (sprite/frame animation/character animation)
/// - Shorthand names with recursive search
/// - Relative paths from the textures directory
///
/// 这是一个路径字符串的简单封装，支持：
/// - 自动类型检测（精灵/帧动画/角色动画）
/// - 支持递归搜索的简写名称
/// - 相对于纹理目录的相对路径
///
/// ## Examples
///
/// ```ron
/// // In bullet configs:
/// visual: "spear"                    // Simple name
/// visual: "battle/bullets/spear"     // Relative path
/// visual: "character_pellet"            // Directory = frame animation
/// visual: "boss.character.ron"       // Character animation
///
/// // In view configs:
/// sprite: (
///     visual: "battle/ui/hpname",
///     transform: (...),
/// )
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct Visual(pub String);

impl Visual {
    /// Create a new Visual from a path string.
    ///
    /// 从路径字符串创建新的 Visual。
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Get the path string.
    ///
    /// 获取路径字符串。
    pub fn path(&self) -> &str {
        &self.0
    }

    /// Resolve this visual path to a typed resource location.
    ///
    /// 将此视觉路径解析为类型化的资源位置。
    pub fn resolve(&self, mod_name: &str) -> Option<ResolvedVisual> {
        resolve_visual_path(&self.0, mod_name)
    }

    /// Check if the path is empty.
    ///
    /// 检查路径是否为空。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for Visual {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Visual {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Visual {
    fn as_ref(&self) -> &str {
        &self.0
    }
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

    /// Frame animation (directory containing multiple images).
    ///
    /// 帧动画（包含多个图片的目录）。
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
/// 2. If path points to a file → Sprite
/// 3. If path points to a directory with 2+ images → FrameAnimation
/// 4. If no extension, search for matching file/directory
///
/// 1. 如果路径以 `.character.ron` 结尾 → CharacterAnimation
/// 2. 如果路径指向文件 → Sprite
/// 3. 如果路径指向包含2+图片的目录 → FrameAnimation
/// 4. 如果没有扩展名，搜索匹配的文件/目录
pub fn resolve_visual_path(input: &str, _mod_name: &str) -> Option<ResolvedVisual> {
    let config = load_config();
    let category_dir = &config.resources.textures;

    // Character animation files are identified by extension
    if input.ends_with(".character.ron") {
        let full_path = resource_resolver::build_resource_path(category_dir, input);
        if full_path.exists() {
            return Some(ResolvedVisual::CharacterAnimation(full_path));
        }
        if let Some(found) = resource_resolver::search_files(category_dir, input, &[]) {
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
        let full_path = resource_resolver::build_resource_path(category_dir, input);
        if full_path.exists() {
            return Some(ResolvedVisual::Sprite(full_path));
        }
    }

    // Try to find as a directory (frame animation)
    let dir_path = resource_resolver::build_resource_path(category_dir, input);
    if dir_path.is_dir() && is_frame_animation_directory(&dir_path) {
        return Some(ResolvedVisual::FrameAnimation(dir_path));
    }

    // Try adding common extensions
    for ext in IMAGE_EXTENSIONS {
        let with_ext = format!("{input}.{ext}");
        let full_path = resource_resolver::build_resource_path(category_dir, &with_ext);
        if full_path.exists() {
            return Some(ResolvedVisual::Sprite(full_path));
        }
    }

    // If simple name (no path separator), search recursively
    if !input.contains('/') && !input.contains('\\') {
        let result = resource_resolver::search_all(category_dir, input, &[]);
        // Prioritize file matches
        if let Some(found) = result.files.first() {
            if result.files.len() > 1 {
                warn!(
                    "Multiple file matches found for '{}': {:?}. Using first match.",
                    input, result.files
                );
            }
            return Some(ResolvedVisual::Sprite(found.clone()));
        }
        // Then check directory matches for frame animation
        if let Some(found) = result
            .dirs
            .into_iter()
            .find(|dir| is_frame_animation_directory(dir))
        {
            return Some(ResolvedVisual::FrameAnimation(found));
        }
    }

    None
}

/// Resolve a visual path using the provided resource paths configuration.
///
/// 使用提供的资源路径配置解析视觉路径。
pub fn resolve_visual_path_with_resources(
    input: &str,
    mod_name: &str,
    _resources: &crate::config::ResourcePaths,
) -> Option<ResolvedVisual> {
    resolve_visual_path(input, mod_name)
}

/// Get the relative path for asset loading from a resolved visual.
///
/// 从解析的视觉获取用于资源加载的相对路径。
pub fn get_asset_path(resolved: &ResolvedVisual, _mod_name: &str) -> String {
    let path = match resolved {
        ResolvedVisual::Sprite(p) => p,
        ResolvedVisual::FrameAnimation(p) => p,
        ResolvedVisual::CharacterAnimation(p) => p,
    };

    resource_resolver::to_relative_asset_path(path)
}

/// Check if a directory looks like a frame animation directory.
/// Frame animation directories should contain multiple images (2 or more).
///
/// 检查目录是否看起来像帧动画目录。
/// 帧动画目录应包含多个图片（2个或更多）。
fn is_frame_animation_directory(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };

    let mut image_count = 0;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && IMAGE_EXTENSIONS.contains(&ext)
        {
            image_count += 1;
        }
    }

    image_count >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_basic() {
        let v1 = Visual::new("heart");
        assert_eq!(v1.path(), "heart");
        assert!(!v1.is_empty());

        let v2: Visual = "spear".into();
        assert_eq!(v2.path(), "spear");

        let v3 = Visual::default();
        assert!(v3.is_empty());
    }

    #[test]
    fn test_visual_deserialize() {
        let ron_str = r#""heart""#;
        let v: Visual = ron::from_str(ron_str).unwrap();
        assert_eq!(v.path(), "heart");
    }
}
