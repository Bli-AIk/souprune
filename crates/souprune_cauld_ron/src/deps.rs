//! Mod dependency resolution from mod.toml files.
//!
//! 从 mod.toml 文件解析 mod 依赖。

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct ModManifest {
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

/// Resolve the dependency graph for a mod and return topological order.
///
/// Reads `projects/<mod_name>/mod.toml`, recursively resolves all transitive
/// dependencies, and returns them in build order (dependencies first, then the
/// target mod itself).
pub fn resolve_deps(projects_root: &Path, mod_name: &str) -> Result<Vec<String>> {
    let mut finished = HashSet::new();
    let mut order = Vec::new();

    visit(
        projects_root,
        mod_name,
        &mut finished,
        &mut BTreeSet::new(),
        &mut order,
    )?;

    Ok(order)
}

fn visit(
    projects_root: &Path,
    mod_name: &str,
    finished: &mut HashSet<String>,
    path: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) -> Result<()> {
    if finished.contains(mod_name) {
        return Ok(());
    }

    if !path.insert(mod_name.to_owned()) {
        let cycle: Vec<String> = path.iter().cloned().collect();
        return Err(anyhow!(
            "circular dependency detected: {}",
            cycle.join(" -> ")
        ));
    }

    let manifest = read_manifest(projects_root, mod_name)?;

    for dep_name in manifest.dependencies.keys() {
        visit(projects_root, dep_name, finished, path, order)?;
    }

    path.remove(mod_name);
    finished.insert(mod_name.to_owned());
    order.push(mod_name.to_owned());

    Ok(())
}

fn read_manifest(projects_root: &Path, mod_name: &str) -> Result<ModManifest> {
    let manifest_path = projects_root.join(mod_name).join("mod.toml");

    if !manifest_path.exists() {
        return Err(anyhow!(
            "mod '{}' not found: {} does not exist",
            mod_name,
            manifest_path.display()
        ));
    }

    let contents = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read mod manifest: {}", manifest_path.display()))?;

    let manifest: ModManifest = toml::from_str(&contents)
        .with_context(|| format!("failed to parse mod manifest: {}", manifest_path.display()))?;

    Ok(manifest)
}
