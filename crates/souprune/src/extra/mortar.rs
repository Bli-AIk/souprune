use bevy::prelude::*;
use bevy_mortar_bond::{MortarAsset, MortarPlugin};
use serde_json::Value;
use std::collections::HashMap;

const UI_TEXT_TEMPLATE: &str = "locales/{locale}/overworld/ui.mortar";

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
struct LocaleMortarHandle {
    handle: Handle<MortarAsset>,
    loaded: bool,
}

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

fn locale_path(locale: &str) -> String {
    UI_TEXT_TEMPLATE.replace("{locale}", locale)
}

fn load_locale_mortar_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    locale: Res<CurrentLocale>,
) {
    let handle: Handle<MortarAsset> = asset_server.load(locale_path(&locale.0));
    commands.insert_resource(LocaleMortarHandle {
        handle,
        loaded: false,
    });
}

fn read_locale_constants_system(
    handle_res: Option<ResMut<LocaleMortarHandle>>,
    assets: Res<Assets<MortarAsset>>,
    mut table: ResMut<MortarStringTable>,
) {
    let Some(mut handle) = handle_res else {
        return;
    };

    if handle.loaded {
        return;
    }

    let Some(asset) = assets.get(&handle.handle) else {
        return;
    };

    for constant in &asset.data.constants {
        if !constant.public {
            continue;
        }

        if let Value::String(value) = &constant.value {
            table.values.insert(constant.name.clone(), value.clone());
        }
    }

    handle.loaded = true;
}
