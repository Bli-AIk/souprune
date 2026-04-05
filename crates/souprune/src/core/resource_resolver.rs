//! # resource_resolver.rs
//!
//! # 资源解析器模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Unified resource path resolution for all asset categories (textures, audios, fonts, etc.).
//! Replaces duplicated resolution logic across visual.rs and audio.rs with a shared interface.
//!
//! 统一的资源路径解析器，适用于所有资源类别（纹理、音频、字体等）。
//! 用共享接口替代 visual.rs 和 audio.rs 中重复的解析逻辑。
//!
//! ## Path Resolution
//!
//! ## 路径解析
//!
//! - Relative path: `"battle/bullets/spear"` → relative to category directory
//! - Simple name: `"heart"` → recursive search in category directory
//! - Extension optional: automatically tries supported extensions
//! - Cascade: searches main mod first, then dependency mods in order
//!
//! - 相对路径：`"battle/bullets/spear"` → 相对于类别目录
//! - 简单名称：`"heart"` → 在类别目录中递归搜索
//! - 扩展名可选：自动尝试支持的扩展名
//! - 级联查找：先搜索主 mod，再按顺序搜索依赖 mod

use std::path::{Path, PathBuf};
use tracing::warn;

/// Build the full path for a resource in a given category directory.
/// Searches main mod first, then dependencies.
///
/// 在给定类别目录下构建资源的完整路径。
/// 先搜索主 mod，再搜索依赖。
pub fn build_resource_path(category_dir: &str, relative: &str) -> PathBuf {
    let config = crate::config::load_config();
    let base = crate::config::get_projects_base_path();

    let primary = base
        .join(&config.project.mod_name)
        .join(category_dir)
        .join(relative);
    if primary.exists() {
        return primary;
    }

    for dep in &config.resolved_dependencies {
        let dep_path = base.join(&dep.name).join(category_dir).join(relative);
        if dep_path.exists() {
            return dep_path;
        }
    }

    primary
}

/// Get all root directories for a given resource category.
/// Returns roots for main mod + dependencies, in priority order.
///
/// 获取给定资源类别的所有根目录。
/// 按优先级返回主 mod + 依赖的根目录。
pub fn all_category_roots(category_dir: &str) -> Vec<PathBuf> {
    let config = crate::config::load_config();
    let base = crate::config::get_projects_base_path();

    let mut roots = vec![base.join(&config.project.mod_name).join(category_dir)];
    for dep in &config.resolved_dependencies {
        roots.push(base.join(&dep.name).join(category_dir));
    }
    roots
}

/// Resolve a resource name to its full filesystem path.
/// Tries: exact path → with extensions → recursive search.
///
/// 将资源名称解析为完整的文件系统路径。
/// 尝试：精确路径 → 添加扩展名 → 递归搜索。
pub fn resolve_resource(category_dir: &str, name: &str, extensions: &[&str]) -> Option<PathBuf> {
    let has_extension = Path::new(name)
        .extension()
        .map(|e| extensions.contains(&e.to_str().unwrap_or("")))
        .unwrap_or(false);

    if has_extension {
        let full_path = build_resource_path(category_dir, name);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    for ext in extensions {
        let with_ext = format!("{name}.{ext}");
        let full_path = build_resource_path(category_dir, &with_ext);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    if !name.contains('/') && !name.contains('\\') {
        let result = search_files(category_dir, name, extensions);
        if result.is_some() {
            return result;
        }
    }

    None
}

/// Search results containing both file and directory matches.
///
/// 包含文件和目录匹配结果的搜索结果。
pub struct SearchResult {
    pub files: Vec<PathBuf>,
    pub dirs: Vec<PathBuf>,
}

/// Recursively search for files matching a stem name within category directories.
/// Returns the first file match found.
///
/// 在类别目录中递归搜索匹配文件名的文件。
/// 返回找到的第一个文件匹配。
pub fn search_files(category_dir: &str, name: &str, extensions: &[&str]) -> Option<PathBuf> {
    let result = search_all(category_dir, name, extensions);
    if result.files.len() > 1 {
        warn!(
            "Multiple file matches found for '{}': {:?}. Using first match.",
            name, result.files
        );
    }
    result.files.into_iter().next()
}

/// Recursively search for files and directories matching a stem name.
/// Used by the visual system which needs directory matches for frame animations.
///
/// 递归搜索匹配文件名的文件和目录。
/// 视觉系统需要目录匹配来识别帧动画。
pub fn search_all(category_dir: &str, name: &str, extensions: &[&str]) -> SearchResult {
    let roots = all_category_roots(category_dir);
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for root in &roots {
        if root.exists() {
            search_recursive_inner(root, name, extensions, &mut files, &mut dirs);
        }
    }

    SearchResult { files, dirs }
}

fn search_recursive_inner(
    dir: &Path,
    name: &str,
    extensions: &[&str],
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

        if stem == name {
            if path.is_dir() {
                dir_results.push(path.clone());
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if extensions.is_empty() || extensions.contains(&ext) {
                    file_results.push(path.clone());
                    continue;
                }
            }
        }

        if path.is_dir() && !file_name.starts_with('.') {
            search_recursive_inner(&path, name, extensions, file_results, dir_results);
        }
    }
}

/// Convert a full filesystem path to a relative asset path by stripping the mod prefix.
/// Used for `AssetServer::load()` which needs mod-relative paths.
///
/// 将完整文件系统路径转换为相对资源路径，去掉 mod 前缀。
/// 用于 `AssetServer::load()`，需要 mod 相对路径。
pub fn to_relative_asset_path(full_path: &Path) -> String {
    let config = crate::config::load_config();
    let base = crate::config::get_projects_base_path();
    let path_str = full_path.to_string_lossy();

    let main_prefix = base
        .join(&config.project.mod_name)
        .to_string_lossy()
        .to_string();
    if let Some(stripped) = path_str.strip_prefix(&main_prefix) {
        return stripped
            .trim_start_matches('/')
            .trim_start_matches('\\')
            .to_string();
    }

    for dep in &config.resolved_dependencies {
        let dep_prefix = base.join(&dep.name).to_string_lossy().to_string();
        if let Some(stripped) = path_str.strip_prefix(&dep_prefix) {
            return stripped
                .trim_start_matches('/')
                .trim_start_matches('\\')
                .to_string();
        }
    }

    path_str.to_string()
}
