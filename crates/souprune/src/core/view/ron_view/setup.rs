use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;

use super::super::components::ViewAnimationState;
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

/// Setup system for ShaderMaterial entities.
/// Creates DynamicMaterial2d assets and attaches Mesh2d and MeshDynamicMaterial2d components.
///
/// ShaderMaterial 实体的设置系统。
/// 创建 DynamicMaterial2d 资产并附加 Mesh2d 和 MeshDynamicMaterial2d 组件。
pub fn setup_shader_materials_system(
    mut commands: Commands,
    procedural_textures: Option<Res<super::super::procedural_textures::ProceduralTextures>>,
    mut dynamic_materials: ResMut<Assets<crate::core::view::dynamic_material::DynamicMaterial2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(
        Entity,
        &super::super::components::ShaderMaterial,
        &super::super::reconcile::ShaderMaterialPendingSetup,
    )>,
) {
    use crate::core::view::dynamic_material::{DynamicMaterial2d, MeshDynamicMaterial2d};

    let Some(textures) = procedural_textures else {
        return;
    };

    if query.is_empty() {
        return;
    }

    // Create quad mesh (unit square, will be scaled by Transform)
    let mesh = meshes.add(Rectangle::new(1.0, 1.0));

    for (entity, shader_mat, pending) in query.iter() {
        // If texture handle is default (placeholder for procedural), use white_pixel
        // 如果纹理句柄是默认的（procedural 的占位符），使用 white_pixel
        let texture = if pending.texture == Handle::default() {
            textures.white_pixel.clone()
        } else {
            pending.texture.clone()
        };

        // Create the DynamicMaterial2d asset
        let material = DynamicMaterial2d {
            shader: shader_mat.shader.clone(),
            params: shader_mat.pack_params(),
            extra_params: shader_mat.pack_extra_params(),
            texture: Some(texture),
        };

        let material_handle = dynamic_materials.add(material);

        // Remove pending marker and add mesh/material components
        commands
            .entity(entity)
            .remove::<super::super::reconcile::ShaderMaterialPendingSetup>()
            .insert((Mesh2d(mesh.clone()), MeshDynamicMaterial2d(material_handle)));

        info!(
            "[ShaderMaterial Setup] Set up dynamic material for entity {:?} with shader {:?}",
            entity, shader_mat.shader
        );
    }
}
