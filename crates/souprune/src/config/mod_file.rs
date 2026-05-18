//! Parses and applies project `mod.toml` files.
//!
//! Keeps the project-specific overlay and dependency resolution logic separate
//! from the top-level runtime configuration types.
//!
//! 解析并应用项目 `mod.toml` 文件。
//! 将项目覆盖配置与依赖解析逻辑从顶层运行时配置类型中拆分出来。

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{error, info};

use super::{FontLayoutConfig, ModeConfig, ResolvedDependency, SoupruneConfig};

#[derive(Deserialize)]
pub(super) struct ModConfigFile {
    pub(super) game: Option<ModGameConfig>,
    #[serde(default)]
    pub(super) resources: Option<ResourcePathsPartial>,
    #[serde(default)]
    pub(super) font_layout: Option<HashMap<String, FontLayoutConfig>>,
    #[serde(default)]
    pub(super) mod_library: Option<ModLibraryConfigPartial>,
    #[serde(default)]
    pub(super) content_library: Option<ContentLibraryConfigPartial>,
    #[serde(default)]
    pub(super) dependencies: HashMap<String, String>,
}

#[derive(Deserialize, Default)]
pub(super) struct ModLibraryConfigPartial {
    wasm: Option<String>,
}

#[derive(Deserialize, Default)]
pub(super) struct ContentLibraryConfigPartial {
    wasm: Option<String>,
    generated_file_header: Option<String>,
}

#[derive(Deserialize, Default)]
pub(super) struct ResourcePathsPartial {
    textures: Option<String>,
    audios: Option<String>,
    fonts: Option<String>,
}

/// Overlay struct for `[game]` in `mod.toml`.
/// All fields are `Option` so that missing entries do not overwrite runtime defaults.
///
/// `mod.toml` 中 `[game]` 节的覆盖结构体。
/// 所有字段均为 `Option`，缺失项不会覆盖运行时默认值。
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(super) struct ModGameConfig {
    pub(super) global_rules: Option<String>,
    pub(super) player_behavior_path: Option<String>,
    pub(super) input_config_path: Option<String>,
    pub(super) flow_config_path: Option<String>,
    pub(super) dialogue_config_path: Option<String>,
    pub(super) chase_config: Option<String>,
    pub(super) dialogue_view_default: Option<String>,
    pub(super) dialogue_voice_default: Option<String>,
    pub(super) enemy_directory: Option<String>,
    pub(super) item_directory: Option<String>,
    pub(super) locales_directory: Option<String>,
    pub(super) required_modules: Option<Vec<String>>,
    pub(super) hidden_layer_keywords: Option<Vec<String>>,
    pub(super) initial_mode: Option<String>,
    pub(super) modes: Option<HashMap<String, ModeConfig>>,
    pub(super) rng_seed: Option<u64>,
}

pub(super) fn read_mod_config<P: AsRef<Path>>(path: P) -> Result<ModConfigFile> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read mod config file at {}", path_ref.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse mod config file at {}", path_ref.display()))
}

/// Apply parsed mod config onto the main config, merging partial fields.
pub(super) fn apply_mod_config(config: &mut SoupruneConfig, mod_cfg: ModConfigFile) {
    #[cfg(target_os = "android")]
    eprintln!(
        "[SoupRune] mod.toml parsed, game section: {:?}",
        mod_cfg.game.is_some()
    );

    if let Some(g) = mod_cfg.game {
        #[cfg(target_os = "android")]
        eprintln!(
            "[SoupRune] game_partial.input_config_path: {:?}",
            g.input_config_path
        );
        macro_rules! merge {
            ($field:ident) => {
                if let Some(val) = g.$field {
                    config.game.$field = val;
                }
            };
        }
        merge!(global_rules);
        merge!(player_behavior_path);
        merge!(input_config_path);
        merge!(flow_config_path);
        merge!(dialogue_config_path);
        merge!(dialogue_view_default);
        merge!(dialogue_voice_default);
        merge!(enemy_directory);
        merge!(item_directory);
        merge!(locales_directory);
        merge!(required_modules);
        merge!(hidden_layer_keywords);
        merge!(initial_mode);
        if let Some(modes) = g.modes {
            merge_mode_configs(&mut config.game.modes, modes);
        }
        if let Some(val) = g.chase_config {
            config.game.chase_config = Some(val);
        }
        if let Some(val) = g.rng_seed {
            config.game.rng_seed = Some(val);
        }
    }
    // Load resource paths from [resources] section (required)
    if let Some(res_partial) = mod_cfg.resources {
        if let Some(val) = res_partial.textures {
            config.resources.textures = val;
        }
        if let Some(val) = res_partial.audios {
            config.resources.audios = val;
        }
        if let Some(val) = res_partial.fonts {
            config.resources.fonts = val;
        }
    }
    if let Some(font_layout) = mod_cfg.font_layout {
        config.font_layout.extend(font_layout);
    }
    // Load mod library configuration from [mod_library] section
    if let Some(lib_partial) = mod_cfg.mod_library
        && let Some(val) = lib_partial.wasm
    {
        config.mod_library.wasm = val;
    }
    if let Some(content_partial) = mod_cfg.content_library {
        if let Some(val) = content_partial.wasm {
            config.content_library.wasm = val;
        }
        if let Some(val) = content_partial.generated_file_header {
            config.content_library.generated_file_header = Some(val);
        }
    }

    // Validate required resource paths
    if config.resources.textures.is_empty() {
        error!("mod.toml: [resources].textures is required");
    }
    if config.resources.audios.is_empty() {
        error!("mod.toml: [resources].audios is required");
    }
    // Fonts default to "assets/fonts" when not specified
    if config.resources.fonts.is_empty() {
        config.resources.fonts = "assets/fonts".to_string();
    }
}

fn merge_mode_configs(
    target: &mut HashMap<String, ModeConfig>,
    incoming: HashMap<String, ModeConfig>,
) {
    for (mode_id, mode_config) in incoming {
        target
            .entry(mode_id)
            .and_modify(|existing| merge_mode_config(existing, mode_config.clone()))
            .or_insert(mode_config);
    }
}

fn merge_mode_config(target: &mut ModeConfig, incoming: ModeConfig) {
    if !incoming.primitives.is_empty() {
        target.primitives = incoming.primitives;
    }
    if incoming.entry_sequence.is_some() {
        target.entry_sequence = incoming.entry_sequence;
    }
    target.rules.extend(incoming.rules);
    if incoming.fixed_camera_zoom.is_some() {
        target.fixed_camera_zoom = incoming.fixed_camera_zoom;
    }
    if incoming.alight_motion_config.is_some() {
        target.alight_motion_config = incoming.alight_motion_config;
    }
}

/// Resolve mod dependencies by reading each dependency's mod.toml.
/// Returns a flat list of dependencies (no transitive resolution yet).
///
/// 通过读取每个依赖的 mod.toml 解析 mod 依赖。
/// 返回扁平的依赖列表（暂无传递依赖解析）。
pub(super) fn resolve_dependencies(
    dependencies: &HashMap<String, String>,
    projects_base: &Path,
) -> (Vec<ResolvedDependency>, Vec<ModConfigFile>) {
    let mut resolved = Vec::new();
    let mut dep_configs = Vec::new();

    for (dep_name, dep_version) in dependencies {
        let dep_dir = projects_base.join(dep_name);
        let dep_mod_toml = dep_dir.join("mod.toml");

        if !dep_mod_toml.exists() {
            error!(
                "Dependency '{}' v{} not found at {}",
                dep_name,
                dep_version,
                dep_mod_toml.display()
            );
            continue;
        }

        match read_mod_config(&dep_mod_toml) {
            Ok(dep_cfg) => {
                let wasm = dep_cfg
                    .mod_library
                    .as_ref()
                    .and_then(|lib| lib.wasm.clone())
                    .unwrap_or_else(|| format!("{dep_name}.wasm"));

                info!(
                    "Resolved dependency: {} v{} (wasm: {})",
                    dep_name, dep_version, wasm
                );
                resolved.push(ResolvedDependency {
                    name: dep_name.clone(),
                    wasm,
                });
                dep_configs.push(dep_cfg);
            }
            Err(e) => {
                error!("Failed to read dependency '{}' mod.toml: {}", dep_name, e);
                continue;
            }
        };
    }

    (resolved, dep_configs)
}
