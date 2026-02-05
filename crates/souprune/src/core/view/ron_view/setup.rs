use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;

use super::super::components::{HPBarLag, HPBarSprite, ViewAnimationState};
use super::super::layout::ViewLayoutAsset;
use super::parsing::parse_overworld_state;
use super::resources::{GlobalTriggerRule, ViewGlobalTriggerConfig, ViewLayoutHandle};
use crate::core::input::ActionRegistry;
use crate::core::sprite::params::SpriteParams;

/// Load global trigger configuration from view layout.
///
/// 从视图布局加载全局触发器配置。
pub fn load_global_triggers_system(
    view_layout_handle: Option<Res<ViewLayoutHandle>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    mut global_trigger_config: ResMut<ViewGlobalTriggerConfig>,
    action_registry: Res<ActionRegistry>,
    mut last_processed_handle: Local<Option<AssetId<ViewLayoutAsset>>>,
    mut events: MessageReader<AssetEvent<ViewLayoutAsset>>,
) {
    let Some(view_layout_handle) = view_layout_handle else {
        return;
    };

    // Check if asset was modified - reset last_processed_handle to force reload
    //
    // 检查资产是否被修改 - 重置 last_processed_handle 以强制重新加载
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == view_layout_handle.handle.id()
        {
            info!("[Hot Reload] Reloading global triggers config...");
            *last_processed_handle = None;
        }
    }

    if last_processed_handle.as_ref() == Some(&view_layout_handle.handle.id()) {
        return;
    }

    let Some(view_layout) = view_layouts.get(&view_layout_handle.handle) else {
        return;
    };

    *last_processed_handle = Some(view_layout_handle.handle.id());

    if let Some(global_triggers) = &view_layout.global_triggers {
        for (action_str, rules_def) in global_triggers {
            if let Some(action) = action_registry.get(action_str) {
                let mut rules = Vec::new();

                for rule_def in rules_def {
                    if let Some(target_state) = parse_overworld_state(&rule_def.target_state) {
                        let allowed_states = rule_def
                            .allowed_states
                            .as_ref()
                            .map(|states| {
                                states
                                    .iter()
                                    .filter_map(|s| parse_overworld_state(s))
                                    .collect()
                            })
                            .unwrap_or_default();

                        rules.push(GlobalTriggerRule {
                            target_state,

                            sound: rule_def.sound.clone(),

                            allowed_states,
                        });
                    } else {
                        warn!(
                            "Unknown target state '{}' in global triggers",
                            rule_def.target_state
                        );
                    }
                }

                global_trigger_config.triggers.insert(action, rules);
            } else {
                warn!("Unknown action '{}' in global triggers", action_str);
            }
        }

        info!(
            "Loaded global trigger config from RON with {} triggers",
            global_triggers.len()
        );
    }
}

pub fn ui_animation_init_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &crate::core::character_asset::CharacterAnimator,
            &ViewAnimationState,
        ),
        Without<Sprite>,
    >,
    anim_configs: Res<Assets<crate::core::character_asset::AnimationConfigAsset>>,
    mut sprite_params: SpriteParams,
) {
    for (entity, animator, anim_state) in query.iter_mut() {
        let Some(config) = anim_configs.get(&animator.config) else {
            continue;
        };

        let clip_name = if let Some(mapping) = config.states.get(&anim_state.state_name) {
            mapping.get_clip_name(&crate::core::basic_components::Direction::Down)
        } else {
            warn!(
                "State {} not found in animation config for UI entity {:?}",
                anim_state.state_name, entity
            );
            continue;
        };

        let clip = match crate::core::animation::components::SpriteAnimationClip::new(
            &mut sprite_params.create_sprite_context(),
            &config.sprite_source,
            clip_name,
        ) {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Failed to load initial UI animation clip {}: {}. Using fallback.",
                    clip_name, e
                );
                crate::core::animation::components::SpriteAnimationClip::fallback(
                    &mut sprite_params.create_sprite_context(),
                    &config.sprite_source,
                    clip_name,
                )
            }
        };

        let sprite = clip.get_current_sprite().clone();
        let frame_duration = sprite_params
            .create_sprite_context()
            .get_animation_frame_duration(clip.clip_name());

        commands.entity(entity).insert((
            sprite,
            clip,
            crate::core::animation::components::SpriteAnimationCurrentFrame::default(),
            crate::core::animation::components::SpriteAnimationTimer::new(frame_duration),
        ));
    }
}

#[allow(clippy::type_complexity)]
pub fn setup_hp_bar_sprites(
    mut commands: Commands,
    procedural_textures: Option<Res<super::super::procedural_textures::ProceduralTextures>>,
    layered_db: Option<Res<bevy_fact_rule_event::LayeredFactDatabase>>,
    mut materials: ResMut<Assets<super::super::custom_sprite_material::CustomSpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // Add Without<Mesh2d> to prevent running every frame
    query: Query<(Entity, &HPBarSprite, &Transform), (Without<Sprite>, Without<Mesh2d>)>,
) {
    use super::super::components::HPSourceType;
    use super::parsing::{PlayerDataView, evaluate_dynamic_color};

    let Some(textures) = procedural_textures else {
        return;
    };

    // Create quad mesh (unit square, will be scaled by Transform)
    let mesh = meshes.add(Rectangle::new(1.0, 1.0));

    // Create a default database for fallback (kept alive for the entire function)
    let default_db = bevy_fact_rule_event::LayeredFactDatabase::default();
    let db_ref: &bevy_fact_rule_event::LayeredFactDatabase = layered_db
        .as_ref()
        .map(|r| r.as_ref())
        .unwrap_or(&default_db);

    let player_data = PlayerDataView::new(db_ref);

    for (entity, hp_bar, _transform) in query.iter() {
        // Get actual HP ratio based on HP source type
        // 根据 HP 来源类型获取实际 HP 比率
        let (actual_hp, actual_hp_max) = match &hp_bar.hp_source {
            HPSourceType::Player => {
                let hp = player_data.get_fact_int("player_hp").unwrap_or(20) as f32;
                let hp_max = player_data.get_fact_int("player_hp_max").unwrap_or(20) as f32;
                (hp, hp_max)
            }
            HPSourceType::Enemy { index } => {
                let hp = player_data
                    .get_fact_int_list("enemy_hps")
                    .and_then(|list| list.get(*index).copied())
                    .unwrap_or(100) as f32;
                let hp_max = player_data
                    .get_fact_int_list("enemy_hp_maxs")
                    .and_then(|list| list.get(*index).copied())
                    .unwrap_or(100) as f32;
                (hp, hp_max)
            }
            HPSourceType::Custom {
                hp_expr,
                hp_max_expr,
            } => {
                // Evaluate custom expressions
                // 计算自定义表达式
                let hp = super::parsing::evaluate_fact_expression(hp_expr, &player_data)
                    .unwrap_or(100.0);
                let hp_max = super::parsing::evaluate_fact_expression(hp_max_expr, &player_data)
                    .unwrap_or(100.0);
                (hp, hp_max)
            }
        };

        let actual_hp_ratio = if actual_hp_max > 0.0 {
            actual_hp / actual_hp_max
        } else {
            1.0
        };

        // Evaluate shader_params from config expressions - config is required
        let (_hp_ratio, _lag_ratio, half_width, alpha) =
            if let Some(ref params) = hp_bar.shader_params_expr {
                evaluate_dynamic_color(params, &player_data, None)
            } else {
                // No config provided - log warning and use minimal fallback
                warn!(
                    "[HP Bar Setup] Entity {:?} missing shader_params config. \
                    Please add shader_params expression in view_layout.ron",
                    entity
                );
                // Use simple fallback values, NOT game-specific formulas
                (1.0, 1.0, 40.0, 1.0)
            };

        // Use actual HP ratio for material initialization, not config values
        // 使用实际 HP 比率进行材质初始化，而不是配置值
        let material = materials.add(super::super::custom_sprite_material::CustomSpriteMaterial {
            color_params: LinearRgba::new(actual_hp_ratio, actual_hp_ratio, half_width, alpha),
            texture: textures.white_pixel.clone(),
        });

        commands.entity(entity).insert((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material),
            HPBarLag::new(actual_hp_ratio),
        ));

        info!(
            "[HP Bar Setup] Spawned HP bar for entity {:?} (source: {:?}). Actual HP ratio: {:.2} ({}/{}), half_width: {:.2}",
            entity, hp_bar.hp_source, actual_hp_ratio, actual_hp, actual_hp_max, half_width
        );
    }
}
