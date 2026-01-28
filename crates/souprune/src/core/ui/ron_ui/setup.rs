use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use std::collections::HashMap;

use super::super::components::{
    HPBarLag, HPBarSprite, IndexBound, LayerTransitions, UIAnimationState, UILayer,
    UILayerNavigationConfig, UILayerNavigationRule, UILayerTransitionConfig,
};
use super::super::layout::{IndexBoundDef, TransitionActionDef, ViewLayoutAsset};
use super::parsing::{parse_action, parse_overworld_state};
use super::resources::{GlobalTriggerRule, UIGlobalTriggerConfig, UILayoutHandle};
use crate::core::sprite::params::SpriteParams;

pub fn load_navigation_and_transitions_system(
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    ui_layouts: Res<Assets<ViewLayoutAsset>>,
    mut navigation_config: ResMut<UILayerNavigationConfig>,
    mut transition_config: ResMut<UILayerTransitionConfig>,
    mut global_trigger_config: ResMut<UIGlobalTriggerConfig>,
    mut last_processed_handle: Local<Option<AssetId<ViewLayoutAsset>>>,
    mut events: MessageReader<AssetEvent<ViewLayoutAsset>>,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    // Check if asset was modified - reset last_processed_handle to force reload
    //
    // 检查资产是否被修改 - 重置 last_processed_handle 以强制重新加载
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == ui_layout_handle.handle.id()
        {
            info!("[Hot Reload] Reloading navigation and transitions config...");
            *last_processed_handle = None;
        }
    }

    if last_processed_handle.as_ref() == Some(&ui_layout_handle.handle.id()) {
        return;
    }

    let Some(ui_layout) = ui_layouts.get(&ui_layout_handle.handle) else {
        return;
    };

    *last_processed_handle = Some(ui_layout_handle.handle.id());

    if let Some(global_triggers) = &ui_layout.global_triggers {
        for (action_str, rules_def) in global_triggers {
            if let Some(action) = parse_action(action_str) {
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

    if let Some(navigation) = &ui_layout.navigation {
        for (layer_name, nav_rule_def) in navigation.iter() {
            let mut adjustments = HashMap::new();

            for (action_str, delta) in &nav_rule_def.mappings {
                if let Some(action) = parse_action(action_str) {
                    adjustments.insert(action, *delta);
                }
            }

            let min_index = nav_rule_def
                .min_index
                .as_ref()
                .map(|bound_def| match bound_def {
                    IndexBoundDef::Static(value) => IndexBound::Static(*value),
                    IndexBoundDef::Dynamic(expr) => IndexBound::Dynamic(expr.clone()),
                });

            let max_index = nav_rule_def
                .max_index
                .as_ref()
                .map(|bound_def| match bound_def {
                    IndexBoundDef::Static(value) => IndexBound::Static(*value),
                    IndexBoundDef::Dynamic(expr) => IndexBound::Dynamic(expr.clone()),
                });

            let layer = UILayer::new(layer_name.clone());
            let rule = UILayerNavigationRule::new_with_bounds(
                adjustments.into_iter(),
                nav_rule_def.looping,
                min_index,
                max_index,
                nav_rule_def.sound_on_navigate.clone(),
            );
            navigation_config.set_rule(layer, rule);
        }
        info!(
            "Loaded navigation config from RON with {} layers",
            navigation.len()
        );
    }

    if let Some(transitions) = &ui_layout.transitions {
        for (layer_name, transitions_def) in transitions.iter() {
            let on_confirm = transitions_def
                .on_confirm
                .as_ref()
                .map(|rules| {
                    rules
                        .iter()
                        .map(|rule_def| {
                            use super::super::components::{TransitionAction, TransitionRule};
                            TransitionRule {
                                condition: rule_def.condition.clone(),
                                action: match &rule_def.action {
                                    TransitionActionDef::GotoLayer(layer) => {
                                        TransitionAction::GotoLayer(UILayer::new(layer.clone()))
                                    }
                                    TransitionActionDef::PopState => TransitionAction::PopState,
                                    TransitionActionDef::PushState(state) => {
                                        TransitionAction::PushState(state.clone())
                                    }
                                },
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let on_cancel = transitions_def.on_cancel.as_ref().map(|action_def| {
                use super::super::components::TransitionAction;
                match action_def {
                    TransitionActionDef::GotoLayer(layer) => {
                        TransitionAction::GotoLayer(UILayer::new(layer.clone()))
                    }
                    TransitionActionDef::PopState => TransitionAction::PopState,
                    TransitionActionDef::PushState(state) => {
                        TransitionAction::PushState(state.clone())
                    }
                }
            });

            let layer = UILayer::new(layer_name.clone());
            transition_config.set_transitions(
                layer,
                LayerTransitions {
                    on_confirm,
                    on_cancel,
                    sound_on_confirm: transitions_def.sound_on_confirm.clone(),
                    sound_on_cancel: transitions_def.sound_on_cancel.clone(),
                },
            );
        }
        info!(
            "Loaded transition config from RON with {} layers",
            transitions.len()
        );
    }
}

pub fn ui_animation_init_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &crate::core::character_asset::CharacterAnimator,
            &UIAnimationState,
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
    player_data: Option<Res<crate::core::data::PlayerData>>,
    mut materials: ResMut<Assets<super::super::custom_sprite_material::CustomSpriteMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // Add Without<Mesh2d> to prevent running every frame
    query: Query<(Entity, &HPBarSprite, &Transform), (Without<Sprite>, Without<Mesh2d>)>,
) {
    let Some(textures) = procedural_textures else {
        return;
    };

    // Use actual player HP if available, otherwise default to full
    let hp_ratio = if let Some(pd) = player_data {
        pd.hp as f32 / pd.hp_max as f32
    } else {
        1.0
    };

    let half_width = 40.0;

    // Create quad mesh (unit square, will be scaled by Transform)
    let mesh = meshes.add(Rectangle::new(1.0, 1.0));

    for (entity, _hp_bar, _transform) in query.iter() {
        let material = materials.add(super::super::custom_sprite_material::CustomSpriteMaterial {
            color_params: LinearRgba::new(hp_ratio, hp_ratio, half_width, 1.0),
            texture: textures.white_pixel.clone(),
        });

        commands.entity(entity).insert((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material),
            HPBarLag::new(hp_ratio),
        ));

        info!(
            "[HP Bar Setup] Spawned HP bar for entity {:?}. Initial HP ratio: {:.2}",
            entity, hp_ratio
        );
    }
}
