//! Registers the View subsystem and wires its runtime pieces into the game schedule.
//!
//! 注册 View 子系统，并把它的运行时部件接入游戏调度。
//!
//! Acts as the plugin front door for Souprune's RON-driven UI runtime. It
//! installs assets, messages, lifecycle tracking, reconciliation, material
//! support, and the system sets that keep views spawning, evaluating, and
//! updating while the game is running.
//!
//! Souprune 的 RON 驱动 UI 运行时的插件入口。它负责安装资源、
//! 消息、生命周期跟踪、对账流程、材质支持，以及一整套让 View 在游戏运行中
//! 能够生成、求值和更新的系统。

use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use bevy_tween::TweenSystemSet;

#[cfg(feature = "debug")]
use super::components::ViewElement;
use super::components::state_sprite::{
    evaluate_new_state_sprites_system, evaluate_state_sprite_rules_system,
    update_state_sprite_textures_system,
};
use super::fact_toggle_color::update_fact_toggle_sdf_colors_system;
use super::layout::ViewLayoutAsset;
use super::lifecycle::{
    StateTransitionTracker, UIInteractiveStateTracker, backpack_state_transition_system,
    cleanup_view_rules_system, process_pending_view_rules_system, state_transition_sound_system,
};
use super::messages::{
    DespawnViewRequest, SpawnViewRequest, handle_despawn_view_request_system,
    handle_spawn_view_request_system,
};
use super::reconcile::ViewReconciliationPlugin;
use super::ron_view::{self, ui_animation_init_system, update_dynamic_text_system};
use super::sdf_view_shape::{sync_view_box_child_visibility_system, update_sdf_view_shape_system};
use super::text::show_text_when_ready_system;
use super::visible_when::evaluate_visible_when_system;
#[cfg(feature = "debug")]
use super::{ElementState, ViewBox, ViewElementHistory, ViewRoot};
use super::{PixelOutlineMaterial, SdfStructureAsset, ViewUpdate};
use crate::core::mode::is_mode;
use crate::core::ron_loader::RonAssetLoader;

/// Asset loader for ViewLayoutAsset that applies coordinate system preprocessing.
///
/// ViewLayoutAsset 的资源加载器，加载后自动应用坐标系预处理。
struct ViewLayoutAssetLoader;

impl bevy::reflect::TypePath for ViewLayoutAssetLoader {
    fn type_path() -> &'static str {
        "souprune::core::view::plugin::ViewLayoutAssetLoader"
    }
    fn short_type_path() -> &'static str {
        "ViewLayoutAssetLoader"
    }
}

impl bevy::asset::AssetLoader for ViewLayoutAssetLoader {
    type Asset = ViewLayoutAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            bevy::asset::io::Reader::read_to_end(reader, &mut bytes).await?;
            let mut asset = ron::de::from_bytes::<ViewLayoutAsset>(&bytes)?;
            asset.apply_coordinate_system();
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["view.ron"]
    }
}

pub struct CoreViewPlugin;

impl Plugin for CoreViewPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.init_asset::<ViewLayoutAsset>()
            .register_asset_loader(ViewLayoutAssetLoader)
            .init_asset::<SdfStructureAsset>()
            .register_asset_loader(RonAssetLoader::<SdfStructureAsset>::new(&["sdf.ron"]))
            .register_type::<super::components::ShaderMaterial>()
            .register_type::<super::components::ViewTextAnimationStyle>()
            .add_plugins(Material2dPlugin::<PixelOutlineMaterial>::default())
            .add_plugins(super::dynamic_material::DynamicMaterial2dPlugin)
            .add_plugins(ViewReconciliationPlugin)
            .init_resource::<UIInteractiveStateTracker>()
            .init_resource::<StateTransitionTracker>()
            .add_message::<SpawnViewRequest>()
            .add_message::<DespawnViewRequest>()
            .add_systems(
                Startup,
                super::procedural_textures::init_procedural_textures,
            )
            .add_systems(
                schedule,
                handle_spawn_view_request_system
                    .after(backpack_state_transition_system)
                    .before(ron_view::spawn_dynamic_view_system),
            )
            .add_systems(schedule, handle_despawn_view_request_system)
            .add_systems(
                schedule,
                (
                    backpack_state_transition_system,
                    state_transition_sound_system,
                )
                    .run_if(is_mode("overworld")),
            )
            .add_systems(
                PostUpdate,
                super::components::box_components::correct_anchored_viewbox_positions
                    .after(TweenSystemSet::ApplyTween),
            )
            .add_systems(
                schedule,
                ron_view::update_shader_materials_system
                    .run_if(resource_exists::<super::procedural_textures::ProceduralTextures>)
                    .after(ron_view::setup_shader_materials_system),
            )
            .add_systems(
                schedule,
                (
                    ron_view::update_time_dependent_ui_elements,
                    ron_view::update_fact_dependent_ui_elements,
                )
                    .run_if(resource_exists::<bevy_fact_rule_event::LayeredFactDatabase>),
            )
            .add_systems(
                schedule,
                update_fact_toggle_sdf_colors_system
                    .after(update_sdf_view_shape_system)
                    .run_if(resource_exists::<bevy_fact_rule_event::LayeredFactDatabase>),
            )
            .add_systems(
                schedule,
                ron_view::reload::validate_map_properties_system.in_set(ViewUpdate),
            )
            .add_systems(
                schedule,
                ron_view::spawn_dynamic_view_system
                    .before(crate::core::fre_bridge::process_view_actions_system)
                    .in_set(ViewUpdate),
            )
            .add_systems(
                schedule,
                (
                    ui_animation_init_system,
                    cleanup_view_rules_system,
                    process_pending_view_rules_system.after(ron_view::spawn_dynamic_view_system),
                )
                    .in_set(ViewUpdate),
            )
            .add_systems(
                schedule,
                (
                    ron_view::setup_shader_materials_system
                        .run_if(resource_exists::<super::procedural_textures::ProceduralTextures>),
                    update_sdf_view_shape_system,
                    show_text_when_ready_system,
                    update_dynamic_text_system,
                    evaluate_state_sprite_rules_system,
                    evaluate_new_state_sprites_system,
                    update_state_sprite_textures_system,
                    evaluate_visible_when_system
                        .run_if(resource_exists::<bevy_fact_rule_event::LayeredFactDatabase>),
                    sync_view_box_child_visibility_system.after(evaluate_visible_when_system),
                )
                    .in_set(ViewUpdate),
            );

        #[cfg(feature = "debug")]
        {
            app.register_type::<ViewBox>()
                .register_type::<ViewElement>()
                .register_type::<ViewRoot>()
                .register_type::<ViewElementHistory>()
                .register_type::<ElementState>();
        }
    }
}
