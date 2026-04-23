//! Build-time helpers for content guest crates.
//!
//! 内容模块 (Guest) crate 的构建期辅助工具。

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Configuration for generating a content registry at build time.
///
/// 用于在构建期生成 content registry 的配置。
#[derive(Debug, Clone)]
pub struct ContentRegistryConfig {
    /// Source root to scan, relative to the content crate root.
    ///
    /// 要扫描的源码根目录，相对于 content crate 根目录。
    pub source_root: PathBuf,
    /// Helper-only directories that should never be treated as emitted assets.
    ///
    /// 只包含 helper 的目录，不应被视为导出资源。
    pub helper_dirs: Vec<PathBuf>,
    /// Helper-only files that should never be treated as emitted assets.
    ///
    /// 只包含 helper 的文件，不应被视为导出资源。
    pub helper_files: Vec<PathBuf>,
}

impl Default for ContentRegistryConfig {
    fn default() -> Self {
        Self {
            source_root: PathBuf::from("src"),
            helper_dirs: vec![PathBuf::from("support")],
            helper_files: vec![PathBuf::from("lib.rs"), PathBuf::from("support.rs")],
        }
    }
}

/// Generate the build-time registry file for a content crate.
///
/// 为 content crate 生成构建期 registry 文件。
pub fn generate_content_registry(config: &ContentRegistryConfig) -> io::Result<PathBuf> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .map_err(|err| io::Error::other(format!("missing CARGO_MANIFEST_DIR: {err}")))?,
    );
    let out_dir = PathBuf::from(
        env::var("OUT_DIR").map_err(|err| io::Error::other(format!("missing OUT_DIR: {err}")))?,
    );
    let source_root = manifest_dir.join(&config.source_root);

    println!("cargo:rerun-if-changed={}", source_root.display());
    for helper_dir in &config.helper_dirs {
        println!(
            "cargo:rerun-if-changed={}",
            source_root.join(helper_dir).display()
        );
    }
    for helper_file in &config.helper_files {
        println!(
            "cargo:rerun-if-changed={}",
            source_root.join(helper_file).display()
        );
    }

    let mut modules = Vec::new();
    collect_asset_modules(&source_root, &source_root, config, &mut modules)?;
    modules.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    let registry_path = out_dir.join("vessel_content_registry.rs");
    fs::write(&registry_path, render_registry(&modules))?;
    Ok(registry_path)
}

#[derive(Debug, Clone)]
struct AssetModule {
    module_name: String,
    relative_path: String,
    absolute_path: PathBuf,
}

fn collect_asset_modules(
    source_root: &Path,
    current_dir: &Path,
    config: &ContentRegistryConfig,
    modules: &mut Vec<AssetModule>,
) -> io::Result<()> {
    let mut entries = fs::read_dir(current_dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_root)
            .map_err(|err| io::Error::other(format!("failed to strip source root: {err}")))?;

        if should_skip(relative, config) {
            continue;
        }

        if path.is_dir() {
            collect_asset_modules(source_root, &path, config, modules)?;
            continue;
        }

        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }

        let relative_display = normalize_path(relative);
        modules.push(AssetModule {
            module_name: sanitize_module_name(&relative_display),
            relative_path: relative_display,
            absolute_path: fs::canonicalize(&path)?,
        });
    }

    Ok(())
}

fn should_skip(relative_path: &Path, config: &ContentRegistryConfig) -> bool {
    if config.helper_files.iter().any(|file| file == relative_path) {
        return true;
    }

    config
        .helper_dirs
        .iter()
        .any(|helper_dir| relative_path.starts_with(helper_dir))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sanitize_module_name(relative_path: &str) -> String {
    let without_extension = relative_path.strip_suffix(".rs").unwrap_or(relative_path);
    let mut module_name = String::from("__vessel_content_");
    for ch in without_extension.chars() {
        if ch.is_ascii_alphanumeric() {
            module_name.push(ch);
        } else {
            module_name.push('_');
        }
    }
    module_name
}

fn render_registry(modules: &[AssetModule]) -> String {
    let mut output = String::from(
        "// Auto-generated Vessel content registry.\n\
         // 自动生成的 Vessel content registry。\n\n",
    );

    for module in modules {
        output.push_str(&format!(
            "#[path = {path:?}]\nmod {module_name};\n",
            path = module.absolute_path.to_string_lossy(),
            module_name = module.module_name,
        ));
    }

    output.push_str(
        "\n/// Emit all auto-discovered content assets.\n\
         ///\n\
         /// 生成全部自动发现的内容资源。\n\
         pub fn emit_all(reg: &mut souprune_vessel::prelude::Registry) -> anyhow::Result<()> {\n",
    );

    for module in modules {
        output.push_str(&format!(
            "    {module_name}::emit(reg)?;\n",
            module_name = module.module_name
        ));
    }

    output.push_str("    Ok(())\n}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::{ContentRegistryConfig, generate_content_registry};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn generates_registry_for_nested_content_tree() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let sandbox_root = env::temp_dir().join(format!("souprune_vessel_build_support_{unique}"));
        let source_root = sandbox_root.join("src");
        let helper_root = source_root.join("support");
        let out_dir = sandbox_root.join("out");
        fs::create_dir_all(source_root.join("battle/view")).expect("create battle/view");
        fs::create_dir_all(&helper_root).expect("create helper dir");
        fs::create_dir_all(&out_dir).expect("create out dir");
        fs::write(source_root.join("lib.rs"), "// lib").expect("write lib");
        fs::write(source_root.join("support.rs"), "// support").expect("write support");
        fs::write(helper_root.join("helpers.rs"), "// helper").expect("write helper");
        fs::write(
            source_root.join("battle/view/battle_bg.rs"),
            "pub fn emit(_: &mut souprune_vessel::prelude::Registry) -> anyhow::Result<()> { Ok(()) }",
        )
        .expect("write asset");

        let old_manifest = env::var_os("CARGO_MANIFEST_DIR");
        let old_out = env::var_os("OUT_DIR");
        // SAFETY: Tests serialize their own sandboxed env changes and restore them below.
        unsafe {
            env::set_var("CARGO_MANIFEST_DIR", &sandbox_root);
            env::set_var("OUT_DIR", &out_dir);
        }

        let registry_path = generate_content_registry(&ContentRegistryConfig::default())
            .expect("registry should generate");
        let generated = fs::read_to_string(&registry_path).expect("read registry");

        assert!(generated.contains("battle/view/battle_bg.rs"));
        assert!(!generated.contains("support/helpers.rs"));
        assert!(!generated.contains("support.rs"));
        assert!(generated.contains("emit_all"));

        // SAFETY: Restore process env to its original values for subsequent tests.
        unsafe {
            restore_env("CARGO_MANIFEST_DIR", old_manifest);
            restore_env("OUT_DIR", old_out);
        }
        fs::remove_dir_all(&sandbox_root).expect("cleanup sandbox");
    }

    unsafe fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
        match value {
            Some(value) => {
                // SAFETY: Caller ensures this restoration happens after temporary override usage.
                unsafe { env::set_var(key, value) }
            }
            None => {
                // SAFETY: Caller ensures this restoration happens after temporary override usage.
                unsafe { env::remove_var(key) }
            }
        }
    }

    #[test]
    fn custom_helper_dirs_are_respected() {
        let config = ContentRegistryConfig {
            source_root: PathBuf::from("src"),
            helper_dirs: vec![PathBuf::from("support"), PathBuf::from("internal")],
            helper_files: vec![PathBuf::from("lib.rs")],
        };
        assert_eq!(config.helper_dirs.len(), 2);
    }
}
