use crate::app_state;
use crate::config;
use crate::core;
use crate::core::input;
use bevy::prelude::*;

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

    if let Some(mut h) = world.get_resource_mut::<app_state::overworld::tilemap::CurrentBgmHandle>()
    {
        h.0 = None;
    }
    if let Some(mut m) = world.get_resource_mut::<app_state::overworld::tilemap::CurrentMapBgm>() {
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
        world.get_resource_mut::<app_state::overworld::trigger::LoadedRuleSets>()
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

    info!("已完成游戏状态重置");
}

pub fn insert_input_resources(app: &mut App) {
    let config = app
        .world()
        .get_resource::<config::SoupruneConfig>()
        .expect("SoupruneConfig must be inserted before calling insert_input_resources");
    let projects_base = config::get_projects_base_path();
    let input_config_path = projects_base
        .join(&config.project.mod_name)
        .join(&config.game.input_config_path);
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
    let config = app
        .world()
        .get_resource::<config::SoupruneConfig>()
        .expect("SoupruneConfig must be inserted before calling insert_font_resources");
    let projects_base = config::get_projects_base_path();
    let font_dir = projects_base
        .join(&config.project.mod_name)
        .join("assets/fonts")
        .to_string_lossy()
        .into_owned();
    app.insert_resource(bevy_bitmap_text::FontDirectories {
        directories: vec![font_dir],
    });
}

pub(crate) fn load_touch_layout(
    input_config: &input::InputConfig,
    projects_base: &std::path::Path,
    mod_name: &str,
) -> Option<input::TouchLayoutDef> {
    let touch_cfg = input_config.touch_overlay.as_ref()?;
    let layout_path = touch_cfg.layout.as_ref()?;
    let full_path = projects_base.join(mod_name).join(layout_path);
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
