use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use serde::Deserialize;

pub struct TomlPlugin;

impl Plugin for TomlPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TomlAsset>()
            .init_asset_loader::<TomlAssetLoader>();
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct TomlAsset {
    pub config: TomlConfig,
}

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    #[serde(default)]
    pub module: Vec<ModuleConfig>,
    #[serde(default)]
    pub animations: Vec<AnimationConfig>,
    #[serde(default)]
    pub sprites: Vec<SpriteConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    #[serde(default)]
    pub loaded_before: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnimationConfig {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default = "default_loop")]
    pub looping: bool,
}

#[derive(Debug, Deserialize)]
pub struct SpriteConfig {
    pub name: String,
    pub path: String,
}

fn default_loop() -> bool {
    true
}

#[derive(Default)]
pub struct TomlAssetLoader;

impl AssetLoader for TomlAssetLoader {
    type Asset = TomlAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let config: TomlConfig = toml::from_str(std::str::from_utf8(&bytes)?)?;

            info!(
                "Successfully loaded toml file: {:?} (animations: {}, sprites: {})",
                load_context.path(),
                config.animations.len(),
                config.sprites.len()
            );

            Ok(TomlAsset { config })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
