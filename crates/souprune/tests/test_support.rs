//! Shared helpers for RON asset tests.
//!
//! RON 资源测试的共享辅助方法。
#![allow(dead_code)]

use once_cell::sync::Lazy;
use ron::de::from_str;
use serde::de::DeserializeOwned;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

static WORKSPACE_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root")
});

static PROJECT_ROOT: Lazy<PathBuf> = Lazy::new(|| WORKSPACE_ROOT.join("projects/example_mod"));
static PROJECT_AM_ROOT: Lazy<PathBuf> =
    Lazy::new(|| WORKSPACE_ROOT.join("projects/example_am_mod"));
static PROJECT_PRESET_ROOT: Lazy<PathBuf> =
    Lazy::new(|| WORKSPACE_ROOT.join("projects/undertale_preset"));

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

/// Path to a named example project folder.
pub fn named_project_root(project_name: &str) -> &'static Path {
    match project_name {
        "example_mod" => PROJECT_ROOT.as_path(),
        "example_am_mod" => PROJECT_AM_ROOT.as_path(),
        "undertale_preset" => PROJECT_PRESET_ROOT.as_path(),
        other => panic!("Unknown test project root: {other}"),
    }
}

/// Path to a named example project folder when that mod checkout is present.
pub fn available_project_root(project_name: &str) -> Option<&'static Path> {
    let root = named_project_root(project_name);
    root.join("mod.toml").is_file().then_some(root)
}

/// Named example projects that are available in the current checkout.
pub fn available_project_names<'a>(project_names: &[&'a str]) -> Vec<&'a str> {
    project_names
        .iter()
        .copied()
        .filter(|project_name| available_project_root(project_name).is_some())
        .collect()
}

/// Emit a visible warning when a smoke test is skipped because optional project repos are absent.
pub fn warn_missing_projects(test_name: &str, detail: &str) {
    let message = format!("{test_name} skipped: {detail}");
    eprintln!("{message}");
    if env::var_os("GITHUB_ACTIONS").is_some() {
        println!("::warning::{message}");
    }
}

/// Parse a project-local RON file relative to the example project root.
///
/// 解析示例项目根目录下的 RON 文件。
pub fn parse_project_ron<T>(relative: &str) -> T
where
    T: DeserializeOwned,
{
    parse_project_ron_from("example_mod", relative)
}

/// Parse a RON file relative to a named project root.
pub fn parse_project_ron_from<T>(project_name: &str, relative: &str) -> T
where
    T: DeserializeOwned,
{
    let path = named_project_root(project_name).join(relative);
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

/// Ensure a relative asset exists under a named project.
pub fn ensure_project_asset_from(project_name: &str, relative: &str) {
    let path = named_project_root(project_name).join(relative);
    if !path.exists() {
        panic!("Asset {} missing", path.display());
    }
}

/// Recursively list project files under `relative_dir` whose file names end with `suffix`.
///
/// 递归列出 `relative_dir` 下以 `suffix` 结尾的项目文件。
pub fn list_project_files_with_suffix(relative_dir: &str, suffix: &str) -> Vec<String> {
    list_project_files_with_suffix_from("example_mod", relative_dir, suffix)
}

/// Recursively list project files for a named project.
pub fn list_project_files_with_suffix_from(
    project_name: &str,
    relative_dir: &str,
    suffix: &str,
) -> Vec<String> {
    let mut files = Vec::new();
    let root = named_project_root(project_name);
    let base = root.join(relative_dir);
    if base.exists() {
        for entry in WalkDir::new(&base)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.to_string_lossy().ends_with(suffix) {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push(relative);
            }
        }
    }
    files.sort();
    files
}
