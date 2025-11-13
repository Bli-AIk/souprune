use crate::app_states::AppState;
use crate::core::camera::Followable;
use crate::core::sprite::ModuleSpriteRegistry;
use bevy::app::{App, Plugin, Update};
use bevy::asset::LoadedFolder;
use bevy::prelude::*;

pub(crate) struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Setup),
            (load_textures_system, setup_camera_system),
        )
        .add_systems(
            Update,
            check_textures_system.run_if(in_state(AppState::Setup)),
        );
    }
}
fn load_textures_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut registry = ModuleSpriteRegistry::new();
    let mut register = (&mut registry, &asset_server);

    // Register for modules here!
    register_module(&mut register, "overworld");
    register_module(&mut register, "battle");

    commands.insert_resource(registry);
}

fn register_module(
    (registry, asset_server): &mut (&mut ModuleSpriteRegistry, &Res<AssetServer>),
    module_name: &str,
) {
    registry.register_module(
        module_name.to_string(),
        asset_server.load_folder(format!("textures/{}", module_name)),
    );
}

fn check_textures_system(
    mut next_state: ResMut<NextState<AppState>>,
    sprite_registry: Res<ModuleSpriteRegistry>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
) {
    // TODO 配置于toml文件
    // 目前会检查所有需要的Sprite是否加载完成，然后才切换状态
    // 但是这样做不够灵活
    // 我们应该在toml文件中配置某个AppState加载前，需要哪些模块的Sprite
    for event in events.read() {
        if let Some(handle) = sprite_registry.get_module("overworld")
            && event.is_loaded_with_dependencies(handle)
        {
            next_state.set(AppState::Overworld);
        }
    }
}

fn setup_camera_system(mut commands: Commands, resolution_scale: Res<ResolutionScale>) {
    commands.spawn((
        Camera2d,
        Transform::from_scale(Vec3::splat(1.0 / resolution_scale.get() as f32)),
        Followable::default(),
    ));
}

#[derive(Resource)]
pub(crate) struct ResolutionScale(pub(crate) u32);

impl ResolutionScale {
    pub(crate) fn get(&self) -> u32 {
        self.0
    }
}

impl Default for ResolutionScale {
    fn default() -> Self {
        // (320, 240) * 2
        Self(5)
    }
}
