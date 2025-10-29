use crate::AppState;
use crate::core::components::Direction;
use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::character::components::*;
use crate::core::sprite::*;
use crate::extra::toml_asset_loader::TomlAsset;
use bevy::app::{App, Plugin};
use bevy::asset::LoadedFolder;
use bevy::ecs::system::SystemParam;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use character::CharacterBundle;
use character::systems::*;
use leafwing_input_manager::action_state::*;
use seldom_state::machine::*;
use seldom_state::trigger::IntoTrigger;
mod character;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Overworld), setup_overworld_system)
            .add_systems(
                Update,
                (
                    update_idle_system,
                    update_walking_system,
                    update_running_system,
                ),
            );
    }
}

struct SpriteLoadContext<'a> {
    sprite_registry: &'a ModuleSpriteRegistry,
    texture_atlases: &'a mut Assets<TextureAtlasLayout>,
    loaded_folders: &'a Assets<LoadedFolder>,
    textures: &'a mut Assets<Image>,
    toml_assets: &'a Assets<TomlAsset>,
    toml_registry: &'a mut TomlConfigRegistry,
}

impl<'a> SpriteLoadContext<'a> {
    fn new(
        sprite_registry: &'a ModuleSpriteRegistry,
        texture_atlases: &'a mut Assets<TextureAtlasLayout>,
        loaded_folders: &'a Assets<LoadedFolder>,
        textures: &'a mut Assets<Image>,
        toml_assets: &'a Assets<TomlAsset>,
        toml_registry: &'a mut TomlConfigRegistry,
    ) -> Self {
        Self {
            sprite_registry,
            texture_atlases,
            loaded_folders,
            textures,
            toml_assets,
            toml_registry,
        }
    }

    fn get_sprite(&mut self, module_name: &str, config_item_name: &str) -> Sprite {
        let handle = self
            .sprite_registry
            .get_module(module_name)
            .unwrap_or_else(|| panic!("{module_name} module not registered"));

        let loaded_folder = self.loaded_folders.get(handle).unwrap();

        let (texture_atlas_nearest, _nearest_sources, nearest_texture, index_map) =
            create_texture_atlas(
                loaded_folder,
                None,
                Some(ImageSampler::nearest()),
                self.textures,
                self.toml_assets,
                self.toml_registry,
                module_name,
            );

        let atlas_nearest_handle = self.texture_atlases.add(texture_atlas_nearest);

        let sprite_path =
            if let Some(sprite_config) = self.toml_registry.get_sprite(config_item_name) {
                sprite_config.path.clone()
            } else {
                panic!("Elf not found in configuration '{}'", config_item_name);
            };

        let sprite_index = *index_map.get(&sprite_path).unwrap_or_else(|| {
            panic!(
                "The path '{}' of sprite '{}' was not found in the gallery",
                config_item_name, sprite_path
            )
        });

        Sprite::from_atlas_image(
            nearest_texture,
            TextureAtlas {
                layout: atlas_nearest_handle.clone(),
                index: sprite_index,
            },
        )
    }
}

#[derive(SystemParam)]
struct SpriteSystemParams<'w> {
    sprite_registry: Res<'w, ModuleSpriteRegistry>,
    texture_atlases: ResMut<'w, Assets<TextureAtlasLayout>>,
    loaded_folders: Res<'w, Assets<LoadedFolder>>,
    textures: ResMut<'w, Assets<Image>>,
    toml_assets: Res<'w, Assets<TomlAsset>>,
    toml_registry: ResMut<'w, TomlConfigRegistry>,
}

impl<'w> SpriteSystemParams<'w> {
    fn create_sprite_context(&mut self) -> SpriteLoadContext<'_> {
        SpriteLoadContext::new(
            &self.sprite_registry,
            &mut self.texture_atlases,
            &self.loaded_folders,
            &mut self.textures,
            &self.toml_assets,
            &mut self.toml_registry,
        )
    }
}

fn setup_overworld_system(
    mut commands: Commands,
    mut sprite_params: SpriteSystemParams,
    player_input: Res<PlayerInputSettings>,
) {
    let sprite = sprite_params
        .create_sprite_context()
        .get_sprite("overworld", "chest_box");

    commands.spawn((
        Idle,
        PlayerControlled,
        StateMachine::default()
            .trans::<Idle, _>(character::is_walking, Walking)
            .trans::<Walking, _>(character::is_walking.not(), Idle)
            .trans::<Running, _>(character::is_walking.not(), Idle)
            .trans::<Walking, _>(character::is_running, Running)
            .trans::<Running, _>(character::is_running.not(), Walking)
            .set_trans_logging(true),
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
        CharacterBundle::new(Vec2::new(0.0, 0.0), Direction::Down, sprite.clone()),
    ));
}
