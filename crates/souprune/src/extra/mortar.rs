use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy_mortar_bond::{MortarAsset, MortarPlugin};
use serde_json::Value;
use std::collections::HashMap;

pub struct MortarExtraPlugin;

impl Plugin for MortarExtraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MortarPlugin)
            .init_resource::<CurrentLocale>()
            .init_resource::<MortarStringTable>()
            .add_systems(Startup, load_locale_mortar_system)
            .add_systems(Update, read_locale_constants_system);
    }
}

#[derive(Resource, Clone)]
pub struct CurrentLocale(pub String);

impl Default for CurrentLocale {
    fn default() -> Self {
        Self("en-US".to_string())
    }
}

#[derive(Resource)]
struct LocalesFolderHandle(Handle<LoadedFolder>);

#[derive(Resource, Default)]
pub struct MortarStringTable {
    values: HashMap<String, String>,
}

impl MortarStringTable {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|value| value.as_str())
    }

    /// Returns the localized string when available, or falls back to the key itself.
    pub fn resolve<'a>(&'a self, name: &'a str) -> &'a str {
        self.get(name).unwrap_or(name)
    }
}

fn load_locale_mortar_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    locale: Res<CurrentLocale>,
) {
    let path = format!("locales/{}", locale.0);
    info!("Loading locales from: {}", path);
    let handle = asset_server.load_folder(path);
    commands.insert_resource(LocalesFolderHandle(handle));
}

#[derive(Resource)]
pub struct LocaleLoaded;

#[allow(clippy::too_many_arguments)]
fn read_locale_constants_system(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    folder_handle: Option<Res<LocalesFolderHandle>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mortar_assets: Res<Assets<MortarAsset>>,
    mut table: ResMut<MortarStringTable>,
    asset_server: Res<AssetServer>,
    locale: Res<CurrentLocale>,
) {
    let Some(folder_handle) = folder_handle else {
        return;
    };

    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event
            && *id == folder_handle.0.id()
        {
            info!("Locales folder loaded. Processing files...");

            if let Some(folder) = loaded_folders.get(&folder_handle.0) {
                for handle in &folder.handles {
                    let id = handle.id();

                    // Determine namespace from path relative to locale folder
                    let namespace = if let Some(path) = asset_server.get_path(id) {
                        let full_path = path.path().to_string_lossy();
                        let prefix = format!("locales/{}/", locale.0);

                        if let Some(remaining) = full_path.strip_prefix(&prefix) {
                            std::path::Path::new(remaining)
                                .with_extension("")
                                .to_string_lossy()
                                .to_string()
                        } else {
                            // Fallback to filename if prefix doesn't match (shouldn't happen if logic is correct)
                            warn!("Path {} does not start with prefix {}", full_path, prefix);
                            path.path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default()
                        }
                    } else {
                        warn!("Could not determine path for asset {:?}", id);
                        continue;
                    };

                    // Try to get as MortarAsset
                    let typed_id = id.typed::<MortarAsset>();
                    if let Some(asset) = mortar_assets.get(typed_id) {
                        info!(
                            "Processing locale file: {}.mortar -> namespace: {}",
                            namespace, namespace
                        );
                        for constant in &asset.data.constants {
                            if !constant.public {
                                continue;
                            }

                            if let Value::String(value) = &constant.value {
                                let key = format!("{}:{}", namespace, constant.name);
                                info!("Registered locale string: {} = {}", key, value);
                                table.values.insert(key, value.clone());
                            }
                        }
                    }
                }
            }
            info!(
                "MortarStringTable initialized. Total strings: {}",
                table.values.len()
            );
            commands.insert_resource(LocaleLoaded);
        }
    }
}
