//! Seeds startup resources and provides the reset helpers needed between runs.
//!
//! 注入启动期资源，并提供切换流程时要用到的运行时重置辅助逻辑。
//!
//! Gathers the pieces that are awkward to keep inside plugins:
//! rebuilding input resources from project config, preparing font directories,
//! loading touch overlay definitions, and clearing stateful runtime data when
//! the game mode is restarted. It is still bootstrap code, but it touches live
//! runtime resources, so the boundary needs to be explicit.
//!
//! 收拢了不适合直接塞进插件装配里的启动辅助逻辑：根据项目配置重建
//! 输入资源、准备字体目录、读取触控布局定义，以及在流程重开时清空有状态的
//! 运行时数据。它仍然属于 bootstrap，但它会直接触碰运行时资源，所以需要把
//! 这层边界写清楚。

use crate::app_state;
use crate::config;
use crate::core;
use crate::core::input;
use bevy::prelude::*;

pub(crate) fn font_layout_overrides_from_config(
    cfg: &config::SoupruneConfig,
) -> bevy_bitmap_text::FontLayoutOverrides {
    let mut overrides = bevy_bitmap_text::FontLayoutOverrides::default();
    for (font_name, layout) in &cfg.font_layout {
        overrides.insert(
            bevy_bitmap_text::FontId::from_name(font_name),
            bevy_bitmap_text::FontLayoutOverride {
                offset_factor: Vec2::new(layout.offset_x_factor, layout.offset_y_factor),
            },
        );
    }
    overrides
}

/// Reset all game runtime state.
pub fn reset_game_state(world: &mut World) {
    if let Some(audio) = world.get_resource::<bevy_kira_audio::Audio>() {
        use bevy_kira_audio::AudioControl;
        audio.stop();
    }
    if let Some(mut instances) = world.get_resource_mut::<Assets<bevy_kira_audio::AudioInstance>>()
    {
        let ids: Vec<_> = instances.ids().collect();
        for id in ids {
            if let Some(instance) = instances.get_mut(id) {
                instance.stop(bevy_kira_audio::AudioTween::default());
            }
            instances.remove(id);
        }
    }

    if let Some(mut bgm) = world.get_resource_mut::<core::sequencer::SequencerBgm>() {
        bgm.handle = None;
        bgm.path = None;
    }

    if let Some(mut h) =
        world.get_resource_mut::<crate::core::overworld::tilemap::CurrentBgmHandle>()
    {
        h.0 = None;
    }
    if let Some(mut m) = world.get_resource_mut::<crate::core::overworld::tilemap::CurrentMapBgm>()
    {
        m.0 = None;
    }

    world.resource_mut::<app_state::SequenceMode>().0 = None;
    world
        .resource_mut::<NextState<app_state::SequenceSubState>>()
        .set(app_state::SequenceSubState::default());

    if let Some(mut ctx) = world.get_resource_mut::<core::sequencer::SequenceContext>() {
        ctx.chapters.clear();
        ctx.state = core::sequencer::SequenceExecutionState::Idle;
    }

    world.remove_resource::<core::sequencer::CurrentSequenceFlow>();
    if let Some(mut srh) = world.get_resource_mut::<core::sequencer::SequenceRulesHandle>() {
        srh.handle = None;
        srh.registered = false;
    }

    if let Some(mut db) = world.get_resource_mut::<bevy_fact_rule_event::LayeredFactDatabase>() {
        db.clear_local();
    }
    if let Some(mut reg) = world.get_resource_mut::<crate::core::game_action::GameRuleRegistry>() {
        reg.clear_local();
    }
    if let Some(mut loaded) =
        world.get_resource_mut::<crate::core::overworld::trigger::LoadedRuleSets>()
    {
        loaded.handles.clear();
        loaded.initialized = false;
        loaded.registered = false;
    }

    let scoped: Vec<Entity> = world
        .query_filtered::<Entity, With<app_state::ModeScoped>>()
        .iter(world)
        .collect();
    for entity in scoped {
        world.despawn(entity);
    }

    let chapters: Vec<Entity> = world
        .query_filtered::<Entity, With<core::sequencer::ActiveChapter>>()
        .iter(world)
        .collect();
    for entity in chapters {
        world.despawn(entity);
    }

    let dialogue_entities: Vec<Entity> = world
        .query_filtered::<Entity, With<core::dialogue::DialogueControllerEntity>>()
        .iter(world)
        .collect();
    for entity in dialogue_entities {
        world.despawn(entity);
    }
    if let Some(mut db) = world.get_resource_mut::<bevy_fact_rule_event::LayeredFactDatabase>() {
        db.set(
            core::fre_facts::DIALOGUE_ACTIVE,
            bevy_fact_rule_event::FactValue::Bool(false),
        );
    }

    let tiled_maps: Vec<Entity> = world
        .query_filtered::<Entity, With<bevy_ecs_tiled::prelude::TiledMap>>()
        .iter(world)
        .collect();
    for entity in tiled_maps {
        world.despawn(entity);
    }

    info!("Game state reset completed!");
}

pub fn insert_input_resources(app: &mut App) {
    let config = app
        .world()
        .get_resource::<config::SoupruneConfig>()
        .expect("SoupruneConfig must be inserted before calling insert_input_resources");
    let projects_base = config::get_projects_base_path();
    let input_config_path =
        config::resolve_path(&config.game.input_config_path).unwrap_or_else(|| {
            projects_base
                .join(&config.project.mod_name)
                .join(&config.game.input_config_path)
        });
    let input_config = input::InputConfig::load_from_file(&input_config_path);
    let action_registry = input_config.build_registry();
    let player_input_settings =
        input::PlayerInputSettings::from_config(&input_config, &action_registry);
    let input_behavior_config = input::InputBehaviorConfig::from_config(&input_config);
    app.insert_resource(action_registry)
        .insert_resource(player_input_settings)
        .insert_resource(input_behavior_config);
}

pub fn insert_font_resources(app: &mut App) {
    let cfg = config::load_config();
    app.insert_resource(bevy_bitmap_text::FontDirectories {
        directories: crate::core::resource_resolver::all_category_roots(&cfg.resources.fonts)
            .iter()
            .filter(|root| root.exists())
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
    })
    .insert_resource(font_layout_overrides_from_config(&cfg));
}

pub(crate) fn load_touch_layout(
    input_config: &input::InputConfig,
    projects_base: &std::path::Path,
    mod_name: &str,
) -> Option<input::TouchLayoutDef> {
    let touch_cfg = input_config.touch_overlay.as_ref()?;
    let layout_path = touch_cfg.layout.as_ref()?;
    let full_path = crate::config::resolve_path(layout_path)
        .unwrap_or_else(|| projects_base.join(mod_name).join(layout_path));
    match input::TouchLayoutDef::load_from_file(&full_path) {
        Ok(mut layout) => {
            info!("Loaded touch layout from {:?}", full_path);
            if let Some(opacity) = touch_cfg.opacity {
                layout.opacity = opacity;
            }
            if let Some(scale) = touch_cfg.scale {
                layout.scale = scale;
            }
            Some(layout)
        }
        Err(e) => {
            warn!("Failed to load touch layout: {}", e);
            None
        }
    }
}
