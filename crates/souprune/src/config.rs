//! Loads the project-level configuration that decides which content Souprune boots and where it reads assets from.
//!
//! 加载项目级配置，决定 Souprune 启动哪套内容，以及资源从哪里读取。
//!
//! This module is the runtime's configuration authority. It reads
//! `projects/config.toml`, combines it with per-mod metadata, and exposes the
//! path helpers that the rest of the framework uses to locate assets, rules, and
//! optional WASM components. If startup wiring needs to know "which project are
//! we running?", the answer comes from here.
//!
//! 这个模块是运行时的配置真源。它读取 `projects/config.toml`，再结合每个
//! mod 自己的元数据，对外提供资源、规则文件以及可选 WASM 组件的路径解析。
//! 启动层如果需要回答“当前运行的是哪一个项目”，答案就来自这里。

use anyhow::{Context, Result};
use bevy::prelude::Resource;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::{error, info};

#[derive(Clone, Deserialize, Resource)]
pub struct SoupruneConfig {
    /// Currently loaded Project (Mod) configuration.
    ///
    /// 当前加载的 Project (Mod) 配置。
    pub project: ProjectConfig,

    /// Window configuration settings.
    ///
    /// 窗口配置设置。
    pub window: WindowConfig,

    /// Game flow configuration settings.
    ///
    /// 游戏流程配置设置。
    #[serde(default)]
    pub game: GameConfig,

    /// Render configuration settings.
    ///
    /// 渲染配置设置。
    #[serde(default)]
    pub render: RenderConfig,

    /// Resource paths configuration.
    ///
    /// 资源路径配置。
    #[serde(skip)]
    pub resources: ResourcePaths,

    /// Per-font layout offsets used by bitmap text rendering.
    ///
    /// 位图文本渲染使用的字体级排版偏移。
    #[serde(default)]
    pub font_layout: HashMap<String, FontLayoutConfig>,

    /// Mod library configuration (WASM component path).
    ///
    /// Mod 库配置（WASM 组件路径）。
    #[serde(skip)]
    pub mod_library: ModLibraryConfig,

    /// Content library configuration for build-time Cauld-ron guests.
    ///
    /// 构建期 Cauld-ron guest 的内容库配置。
    #[serde(skip)]
    pub content_library: ContentLibraryConfig,

    /// Resolved mod dependencies (populated from `[dependencies]` in mod.toml).
    /// Ordered so that transitive dependencies come before direct ones.
    ///
    /// 已解析的 mod 依赖列表（来自 mod.toml 的 `[dependencies]` 节）。
    /// 传递依赖排在直接依赖之前。
    #[serde(skip)]
    pub resolved_dependencies: Vec<ResolvedDependency>,
}

/// A resolved mod dependency with its name and WASM path.
///
/// 已解析的 mod 依赖，包含名称和 WASM 路径。
#[derive(Clone, Debug)]
pub struct ResolvedDependency {
    /// Mod name (matches a directory under `projects/`).
    ///
    /// Mod 名称（对应 `projects/` 下的目录）。
    pub name: String,

    /// WASM component filename from the dependency's mod.toml.
    ///
    /// 依赖的 mod.toml 中的 WASM 组件文件名。
    pub wasm: String,
}

/// WASM mod library configuration from mod.toml [mod_library] section.
///
/// mod.toml 中 [mod_library] 节的 WASM 模组库配置。
#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct ModLibraryConfig {
    /// WASM component filename (e.g., ".build/runtime.wasm").
    ///
    /// WASM 组件文件名（如 ".build/runtime.wasm"）。
    pub wasm: String,
}

/// WASM content library configuration from mod.toml [content_library] section.
///
/// mod.toml 中 [content_library] 节的 WASM 内容库配置。
#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct ContentLibraryConfig {
    /// WASM component filename for the content guest.
    ///
    /// 内容模块 (Guest) 的 WASM 组件文件名。
    pub wasm: String,

    /// Optional file header prepended to generated content files.
    /// When absent, Cauld-ron uses its default bootstrap warning block.
    ///
    /// 生成内容文件时附加的可选文件头。
    /// 未设置时，Cauld-ron 使用默认的 bootstrap 警告块。
    pub generated_file_header: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ProjectConfig {
    pub mod_name: String,
    pub language: String,
}

#[derive(Clone, Deserialize)]
pub struct WindowConfig {
    pub resolution_scale: u32,
}

/// Game flow configuration for paths and module loading.
///
/// 游戏流程配置，包含路径和模块加载设置。
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    /// Path to the global rules file (loaded into FRE Global layer at startup).
    /// This contains initial player data and game-wide facts.
    ///
    /// 全局规则文件路径（启动时加载到 FRE 全局层）。
    /// 包含初始玩家数据和游戏全局事实。
    pub global_rules: String,

    /// Initial sequence path for the Battle state.
    /// When set and `initial_sequence_path` is absent, the game starts directly in Battle mode.
    ///
    /// 战斗状态的初始序列路径。
    /// 当设置此项且 `initial_sequence_path` 未设置时，游戏直接以 Battle 模式启动。
    pub initial_battle_path: String,

    /// Optional sequence path to load when entering Overworld.
    /// When set, the Overworld initialization is driven by this sequence
    /// instead of hardcoded OnEnter systems.
    ///
    /// 进入 Overworld 时加载的可选序列路径。
    /// 设置后，Overworld 的初始化由此序列驱动，而非硬编码的 OnEnter 系统。
    #[serde(default)]
    pub initial_sequence_path: Option<String>,

    /// Path to player behavior configuration file.
    ///
    /// 玩家行为配置文件路径。
    pub player_behavior_path: String,

    /// Path to input configuration file (RON format).
    ///
    /// 输入配置文件路径（RON 格式）。
    pub input_config_path: String,

    /// Path to flow configuration file (RON format).
    ///
    /// 流程配置文件路径（RON 格式）。
    pub flow_config_path: String,

    /// Path to dialogue configuration file (RON format).
    ///
    /// 对话配置文件路径（RON 格式）。
    pub dialogue_config_path: String,

    /// Path to chase state configuration file (RON format).
    /// If None, chase state functionality is disabled.
    ///
    /// 追逐战状态配置文件路径（RON 格式）。
    /// 如果为 None，则禁用追逐战功能。
    pub chase_config: Option<String>,

    /// Default dialogue view layout path for Tiled objects.
    /// Can be overridden per-object via `dialogue_view` property.
    ///
    /// Tiled 对象的默认对话视图布局路径。
    /// 可通过 `dialogue_view` 属性覆盖。
    pub dialogue_view_default: String,

    /// Default dialogue voice audio path for item dialogues.
    /// Controls the typewriter sound effect when item dialogue text is revealed.
    ///
    /// 物品对话的默认打字机音效路径。
    /// 控制物品对话文本逐字显示时的音效。
    #[serde(default)]
    pub dialogue_voice_default: String,

    /// Folder containing enemy definition assets.
    ///
    /// 敌人定义资源目录。
    pub enemy_directory: String,

    /// Folder containing item list assets.
    ///
    /// 物品列表资源目录。
    pub item_directory: String,

    /// Folder containing localized Mortar assets.
    ///
    /// 本地化 Mortar 资源目录。
    pub locales_directory: String,

    /// Texture modules required before transitioning from AppSetup.
    ///
    /// 从 AppSetup 状态转换前需要加载的纹理模块。
    pub required_modules: Vec<String>,

    /// Keywords for layer names that should be hidden (e.g., prototype, collision).
    ///
    /// 需要隐藏的图层名关键字（如 prototype、collision）。
    pub hidden_layer_keywords: Vec<String>,

    /// Initial mode to enter after loading completes.
    /// Determined from config: if `initial_sequence_path` is set, the mode is
    /// inferred from the sequence; otherwise falls back to this value.
    ///
    /// 加载完成后进入的初始模式。
    /// 若 `initial_sequence_path` 已设置，模式从序列中推导；否则使用此值。
    #[serde(default = "default_initial_mode")]
    pub initial_mode: String,

    /// FRE rule files loaded for overworld state (accumulated from dependency chain).
    /// Rules from dependency mods come first, main mod's rules last.
    ///
    /// Overworld 状态加载的 FRE 规则文件（从依赖链累积）。
    /// 依赖 mod 的规则在前，主 mod 的规则在后。
    #[serde(default)]
    pub overworld_rules: Vec<String>,

    /// Camera zoom level applied immediately when entering battle mode.
    /// This avoids the visual delay of setting zoom through the sequencer.
    ///
    /// 进入战斗模式时立即应用的摄像机缩放级别。
    /// 避免通过序列器设置缩放产生的视觉延迟。
    #[serde(default = "default_battle_camera_zoom")]
    pub battle_camera_zoom: f32,

    /// Optional fixed RNG seed for deterministic behavior.
    /// When set, all random operations (enemy turns, RandomPick, etc.) produce
    /// repeatable results — useful for testing and replay.
    ///
    /// 可选的固定 RNG 种子，用于确定性行为。
    /// 设置后，所有随机操作（敌人回合、RandomPick 等）将产生可重现的结果——
    /// 适用于测试和回放。
    #[serde(default)]
    pub rng_seed: Option<u64>,
}

fn default_battle_camera_zoom() -> f32 {
    2.0
}

fn default_initial_mode() -> String {
    "overworld".to_string()
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            global_rules: String::new(),
            initial_battle_path: String::new(),
            initial_sequence_path: None,
            player_behavior_path: String::new(),
            input_config_path: "app/input.ron".to_string(),
            flow_config_path: "app/flow.ron".to_string(),
            dialogue_config_path: "narrative/dialogue.ron".to_string(),
            chase_config: None,
            dialogue_view_default: "overworld/view/dialogue.view.ron".to_string(),
            dialogue_voice_default: "assets/audios/voice/voice_monster.wav".to_string(),
            enemy_directory: "actors/enemies".to_string(),
            item_directory: "actors/items".to_string(),
            locales_directory: "assets/locales".to_string(),
            required_modules: vec!["overworld".to_string(), "common".to_string()],
            hidden_layer_keywords: vec!["prototype".to_string(), "collision".to_string()],
            initial_mode: default_initial_mode(),
            overworld_rules: Vec::new(),
            battle_camera_zoom: default_battle_camera_zoom(),
            rng_seed: None,
        }
    }
}

/// Render configuration for resolution and Z-ordering.
///
/// 渲染配置，包含分辨率和 Z 轴排序设置。
#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    /// Base resolution width (game world units).
    ///
    /// 基准分辨率宽度（游戏世界单位）。
    pub base_resolution_width: u32,

    /// Base resolution height (game world units).
    ///
    /// 基准分辨率高度（游戏世界单位）。
    pub base_resolution_height: u32,

    /// Z-offset applied to tilemap layers.
    ///
    /// 应用于 tilemap 图层的 Z 轴偏移。
    pub z_layer_tilemap: f32,

    /// Base Z value for layer sorting.
    ///
    /// 图层排序的基准 Z 值。
    pub z_layer_base: f32,

    /// Z step between consecutive layers.
    ///
    /// 连续图层之间的 Z 步长。
    pub z_layer_step: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            base_resolution_width: 320,
            base_resolution_height: 240,
            z_layer_tilemap: 10.0,
            z_layer_base: -2.0,
            z_layer_step: 0.5,
        }
    }
}

/// Per-font layout correction for bitmap text.
///
/// 位图文本的字体级排版校正。
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct FontLayoutConfig {
    /// Horizontal glyph offset as a fraction of text world scale.
    ///
    /// 字形水平偏移量，单位为文本世界缩放的比例。
    pub offset_x_factor: f32,

    /// Vertical glyph offset as a fraction of text world scale.
    ///
    /// 字形垂直偏移量，单位为文本世界缩放的比例。
    pub offset_y_factor: f32,
}

static CONFIG: OnceLock<SoupruneConfig> = OnceLock::new();

/// Android private app files root for Souprune.
///
/// Souprune 在 Android 上的私有应用文件根目录。
pub(crate) fn android_private_base_path() -> PathBuf {
    std::env::var_os("SOUPRUNE_PRIVATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune"))
}

/// Returns the base path for the `projects/` directory.
/// On Android, this resolves to private app storage (`.../SoupRune/projects/`).
/// On desktop platforms, this returns the relative `projects/` path.
///
/// 返回 `projects/` 目录的基础路径。
/// 在 Android 上，解析为私有应用存储（`.../SoupRune/projects/`）。
/// 在桌面平台上，返回相对路径 `projects/`。
pub fn get_projects_base_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        android_private_base_path().join("projects")
    }
    #[cfg(not(target_os = "android"))]
    {
        PathBuf::from("projects")
    }
}

pub fn get_asset_roots(mod_name: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let projects_base = get_projects_base_path();
    let project_path = projects_base.join(mod_name);

    // Primary: project's assets directory
    // 主要：项目的 assets 目录
    roots.push(project_path.join("assets"));

    // Also check the project root as a fallback location
    // 同时检查项目根目录作为备选位置
    roots.push(project_path.clone());

    // Fallback to absolute path to ensure assets are found
    // 回退到绝对路径以确保找到资源
    if let Ok(abs_path) = dunce::canonicalize(&project_path) {
        roots.push(abs_path.join("assets"));
        roots.push(abs_path);
    }

    #[cfg(feature = "debug")]
    {
        roots.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(&project_path)
                .join("assets"),
        );
        roots.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(&project_path),
        );
    }

    roots
}

/// Returns all asset roots for the current project and its dependencies.
/// Search priority: current mod first, then dependencies in order.
///
/// 返回当前项目及其依赖的所有资源根目录。
/// 搜索优先级：当前 mod 优先，然后按顺序搜索依赖。
pub fn get_all_asset_roots() -> Vec<PathBuf> {
    let config = load_config();
    let mut all_roots = get_asset_roots(&config.project.mod_name);
    for dep in &config.resolved_dependencies {
        all_roots.extend(get_asset_roots(&dep.name));
    }
    all_roots
}

pub fn resolve_path(relative_path: &str) -> Option<PathBuf> {
    let roots = get_all_asset_roots();

    for root in roots {
        let candidate = root.join(relative_path);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_projects_base_path_stays_relative() {
        assert_eq!(get_projects_base_path(), PathBuf::from("projects"));
    }

    #[test]
    fn android_private_base_path_points_to_app_storage() {
        // SAFETY: These tests do not spawn threads that read this process
        // environment while the variable is being changed.
        unsafe {
            std::env::remove_var("SOUPRUNE_PRIVATE_ROOT");
        }
        assert_eq!(
            android_private_base_path(),
            PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune")
        );
        assert_eq!(
            android_private_base_path().join("projects"),
            PathBuf::from("/data/user/0/com.bliaik.souprune/files/SoupRune/projects")
        );
    }

    #[test]
    fn android_private_base_path_can_come_from_environment() {
        let root = PathBuf::from("/tmp/souprune-private-root-test");
        // SAFETY: These tests do not spawn threads that read this process
        // environment while the variable is being changed.
        unsafe {
            std::env::set_var("SOUPRUNE_PRIVATE_ROOT", &root);
        }
        assert_eq!(android_private_base_path(), root);
        // SAFETY: See safety note above.
        unsafe {
            std::env::remove_var("SOUPRUNE_PRIVATE_ROOT");
        }
    }
}

/// Resource paths configuration from mod.toml [resources] section.
///
/// mod.toml 中 [resources] 节的资源路径配置。
#[derive(Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ResourcePaths {
    /// Path to textures directory relative to mod root.
    ///
    /// 纹理目录路径，相对于 mod 根目录。
    pub textures: String,

    /// Path to audio directory relative to mod root.
    ///
    /// 音频目录路径，相对于 mod 根目录。
    pub audios: String,

    /// Path to fonts directory relative to mod root.
    ///
    /// 字体目录路径，相对于 mod 根目录。
    pub fonts: String,
}

#[derive(Deserialize)]
struct ModConfigFile {
    game: Option<ModGameConfig>,
    #[serde(default)]
    resources: Option<ResourcePathsPartial>,
    #[serde(default)]
    font_layout: Option<HashMap<String, FontLayoutConfig>>,
    #[serde(default)]
    mod_library: Option<ModLibraryConfigPartial>,
    #[serde(default)]
    content_library: Option<ContentLibraryConfigPartial>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

#[derive(Deserialize, Default)]
struct ModLibraryConfigPartial {
    wasm: Option<String>,
}

#[derive(Deserialize, Default)]
struct ContentLibraryConfigPartial {
    wasm: Option<String>,
    generated_file_header: Option<String>,
}

#[derive(Deserialize, Default)]
struct ResourcePathsPartial {
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
#[serde(default)]
struct ModGameConfig {
    global_rules: Option<String>,
    initial_battle_path: Option<String>,
    initial_sequence_path: Option<String>,
    player_behavior_path: Option<String>,
    input_config_path: Option<String>,
    flow_config_path: Option<String>,
    dialogue_config_path: Option<String>,
    chase_config: Option<String>,
    dialogue_view_default: Option<String>,
    dialogue_voice_default: Option<String>,
    enemy_directory: Option<String>,
    item_directory: Option<String>,
    locales_directory: Option<String>,
    required_modules: Option<Vec<String>>,
    hidden_layer_keywords: Option<Vec<String>>,
    initial_mode: Option<String>,
    overworld_rules: Option<Vec<String>>,
    battle_camera_zoom: Option<f32>,
    rng_seed: Option<u64>,
}

fn read_mod_config<P: AsRef<Path>>(path: P) -> Result<ModConfigFile> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read mod config file at {}", path_ref.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse mod config file at {}", path_ref.display()))
}

/// Apply parsed mod config onto the main config, merging partial fields.
fn apply_mod_config(config: &mut SoupruneConfig, mod_cfg: ModConfigFile) {
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
        merge!(initial_battle_path);
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
        merge!(battle_camera_zoom);
        // overworld_rules: extend rather than overwrite (dependency chain accumulation)
        if let Some(val) = g.overworld_rules {
            config.game.overworld_rules.extend(val);
        }
        // Option<T> fields: wrap in Some
        if let Some(val) = g.initial_sequence_path {
            config.game.initial_sequence_path = Some(val);
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

/// Resolve mod dependencies by reading each dependency's mod.toml.
/// Returns a flat list of dependencies (no transitive resolution yet).
///
/// 通过读取每个依赖的 mod.toml 解析 mod 依赖。
/// 返回扁平的依赖列表（暂无传递依赖解析）。
fn resolve_dependencies(
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

fn read_config_from_disk<P: AsRef<Path>>(path: P) -> Result<SoupruneConfig> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read config file at {}", path_ref.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file at {}", path_ref.display()))
}

pub fn load_config() -> SoupruneConfig {
    CONFIG
        .get_or_init(|| {
            let projects_base = get_projects_base_path();
            let config_path = projects_base.join("config.toml");

            let mut config = read_config_from_disk(&config_path).unwrap_or_else(|err| {
                error!(
                    "{}
Falling back to default configuration (mad_dummy_example)",
                    err
                );
                default_config()
            });

            // Initialize resources - will be populated from mod.toml
            config.resources = ResourcePaths::default();

            let mod_name = &config.project.mod_name;
            let mod_config_path = projects_base.join(mod_name).join("mod.toml");

            #[cfg(target_os = "android")]
            eprintln!(
                "[SoupRune] mod_config_path: {:?}, exists: {}",
                mod_config_path,
                mod_config_path.exists()
            );

            if !mod_config_path.exists() {
                return config;
            }

            match read_mod_config(&mod_config_path) {
                Ok(mod_cfg) => {
                    let (deps, dep_configs) =
                        resolve_dependencies(&mod_cfg.dependencies, &projects_base);

                    // Apply dependency configs first (lower priority)
                    for dep_cfg in dep_configs {
                        apply_mod_config(&mut config, dep_cfg);
                    }

                    // Apply main mod config last (highest priority, overwrites)
                    apply_mod_config(&mut config, mod_cfg);
                    config.resolved_dependencies = deps;
                }
                Err(e) => {
                    #[cfg(target_os = "android")]
                    eprintln!("[SoupRune] Failed to load mod.toml: {:#}", e);
                    error!("Failed to load mod.toml: {}", e);
                }
            }

            config
        })
        .clone()
}

fn default_config() -> SoupruneConfig {
    SoupruneConfig {
        project: ProjectConfig {
            mod_name: "mad_dummy_example".to_string(),
            language: "en-US".to_string(),
        },
        window: WindowConfig {
            resolution_scale: 2,
        },
        game: GameConfig::default(),
        render: RenderConfig::default(),
        resources: ResourcePaths::default(),
        font_layout: HashMap::new(),
        mod_library: ModLibraryConfig::default(),
        content_library: ContentLibraryConfig::default(),
        resolved_dependencies: Vec::new(),
    }
}
