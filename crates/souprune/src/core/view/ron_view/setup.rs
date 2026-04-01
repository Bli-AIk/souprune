//! Performs one-time setup work for generated view entities after they are spawned.
//!
//! 在生成 View 实体之后，执行那些只需要做一次的初始化工作。
//!
//! Handles setup that depends on already-created entities and loaded
//! assets, such as attaching initial sprite animation state or turning pending
//! shader-material placeholders into real render components. It is part of the
//! spawn pipeline, but it runs after entity creation rather than during layout parsing.
//!
//! 处理那些依赖“实体已经生成且资源已经可用”的初始化步骤，例如挂上
//! 初始精灵动画状态，或把待处理的 shader 材质占位符变成真正的渲染组件。
//! 它属于生成链的一部分，但发生在实体创建之后，而不是布局解析阶段。

use bevy::prelude::*;

use super::super::components::ViewAnimationState;
use crate::core::sprite::params::SpriteParams;

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

        let entry = if let Some(mapping) = config.states.get(&anim_state.state_name) {
            crate::core::character_asset::state_animation_entry(
                mapping,
                &crate::core::basic_components::Direction::Down,
            )
        } else {
            warn!(
                "State {} not found in animation config for UI entity {:?}",
                anim_state.state_name, entity
            );
            continue;
        };

        let looping = entry
            .looping_override()
            .unwrap_or(config.default_looping);
        let frame_duration = entry
            .frame_duration_override()
            .unwrap_or(config.default_frame_duration);

        let clip = match crate::core::animation::components::SpriteAnimationClip::new(
            &mut sprite_params.create_sprite_context(),
            &config.sprite_source,
            entry.path(),
            entry.flip_x(),
            entry.flip_y(),
            looping,
            frame_duration,
        ) {
            Ok(c) => c,
            Err(e) => {
                error!(
                    "Failed to load initial UI animation clip {}: {}. Using fallback.",
                    entry.path(), e
                );
                crate::core::animation::components::SpriteAnimationClip::fallback(
                    &mut sprite_params.create_sprite_context(),
                    entry.path(),
                    frame_duration,
                )
            }
        };

        let sprite = clip.get_current_sprite().clone();

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
    use crate::core::view::dynamic_material::{
        DynamicMaterial2d, MaterialAssetIdDebug, MeshDynamicMaterial2d,
    };

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
        let asset_id_debug = MaterialAssetIdDebug {
            asset_id: format!("{:?}", material_handle.id()),
        };

        // Remove pending marker and add mesh/material components
        commands
            .entity(entity)
            .remove::<super::super::reconcile::ShaderMaterialPendingSetup>()
            .insert((
                Mesh2d(mesh.clone()),
                MeshDynamicMaterial2d(material_handle),
                asset_id_debug,
            ));
    }
}
