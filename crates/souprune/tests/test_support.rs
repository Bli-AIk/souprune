//! Shared helpers for RON asset tests.
//!
//! RON 资产测试的共享辅助方法。
#![allow(dead_code)]

use once_cell::sync::Lazy;
use ron::de::from_str;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

static WORKSPACE_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root")
});

static PROJECT_ROOT: Lazy<PathBuf> = Lazy::new(|| WORKSPACE_ROOT.join("projects/example_mod"));

/// Read a UTF-8 file as string.
///
/// 读取 UTF-8 文件为字符串。
pub fn read_string(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", path.display()))
}

/// Path to the active example project folder.
///
/// 指向当前示例项目文件夹的路径。
pub fn project_root() -> &'static Path {
    PROJECT_ROOT.as_path()
}

/// Parse a project-local RON file relative to the example project root.
///
/// 解析示例项目根目录下的 RON 文件。
pub fn parse_project_ron<T>(relative: &str) -> T
where
    T: DeserializeOwned,
{
    let path = PROJECT_ROOT.join(relative);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", path.display()));
    from_str(&contents).unwrap_or_else(|err| panic!("Failed to parse {}: {err}", path.display()))
}

/// Ensure a relative project asset exists.
///
/// 确认项目内相对路径存在。
pub fn ensure_project_asset(relative: &str) {
    let path = PROJECT_ROOT.join(relative);
    if !path.exists() {
        panic!("Asset {} missing", path.display());
    }
}
