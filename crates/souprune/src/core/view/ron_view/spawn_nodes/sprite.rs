//! # sprite.rs
//!
//! # sprite.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Handles the sprite-specific branches of the RON View node spawner. It decides
//! whether a sprite should be treated as a protocol texture, a resolved game asset, or a shader
//! material node, and builds the corresponding entity bundle.
//!
//! 负责 RON View 节点生成里与精灵有关的分支。它会判断一个精灵应该按协议纹理、
//! 解析后的游戏资产，还是着色器材质节点来处理，并生成对应的实体组合。

use super::super::parsing::preprocess_sprite_def_for_repeat;
use super::super::resources::RonDrivenView;
use super::super::spawn_helpers::spawn_standalone_static_sprite;
use bevy::prelude::*;

pub(super) fn spawn_standalone_sprite_node(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &crate::core::view::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    visual_path: &str,
    transform: Transform,
    node_name: &str,
    repeat_ctx: Option<&super::super::parsing::RepeatContext>,
) -> Entity {
    if sprite_def.material.is_some() {
        return spawn_material_sprite(
            parent,
            asset_server,
            sprite_def,
            view_element,
            visual_path,
            transform,
            node_name,
            repeat_ctx,
        );
    }

    if visual_path.contains("://") {
        return spawn_protocol_sprite(
            parent,
            asset_server,
            sprite_def,
            view_element,
            visual_path,
            transform,
            node_name,
        );
    }

    use crate::config::load_config;
    use crate::core::visual::{get_asset_path, resolve_visual_path};

    let config = load_config();
    if let Some(resolved) = resolve_visual_path(visual_path, &config.project.mod_name) {
        let asset_path = get_asset_path(&resolved, &config.project.mod_name);
        return spawn_resolved_sprite(
            parent,
            asset_server,
            sprite_def,
            view_element,
            &resolved,
            &asset_path,
            transform,
            node_name,
        );
    }

    let texture_handle = asset_server.load(visual_path.to_owned());
    let mut entity_id = None;
    spawn_standalone_static_sprite(
        parent,
        sprite_def,
        view_element,
        texture_handle,
        transform,
        node_name,
        &mut entity_id,
        visual_path,
    );
    entity_id.expect("spawn_standalone_static_sprite must set entity_id")
}

fn spawn_protocol_sprite(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &crate::core::view::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    visual_path: &str,
    transform: Transform,
    node_name: &str,
) -> Entity {
    let texture_handle: Handle<Image> = if visual_path.starts_with("procedural://") {
        Handle::default()
    } else {
        asset_server.load(visual_path.to_owned())
    };
    let mut entity_id = None;
    spawn_standalone_static_sprite(
        parent,
        sprite_def,
        view_element,
        texture_handle,
        transform,
        node_name,
        &mut entity_id,
        visual_path,
    );
    entity_id.expect("spawn_standalone_static_sprite must set entity_id")
}

fn spawn_resolved_sprite(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &crate::core::view::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    resolved: &crate::core::visual::ResolvedVisual,
    asset_path: &str,
    transform: Transform,
    node_name: &str,
) -> Entity {
    match resolved {
        crate::core::visual::ResolvedVisual::CharacterAnimation(_) => {
            let config_handle = asset_server
                .load::<crate::core::character_asset::AnimationConfigAsset>(asset_path.to_owned());
            let mut entity_cmd = parent.spawn((
                crate::core::character_asset::CharacterAnimator {
                    config: config_handle,
                },
                crate::core::view::components::ViewAnimationState {
                    state_name: sprite_def
                        .initial_state
                        .clone()
                        .unwrap_or("Idle".to_string()),
                },
                transform,
                Visibility::default(),
                Name::new(node_name.to_owned()),
                RonDrivenView,
            ));
            if let Some(view_element) = view_element {
                entity_cmd.insert(view_element.clone());
            }
            info!("[UI Sprite] Spawned animated sprite '{}'", node_name);
            entity_cmd.id()
        }
        crate::core::visual::ResolvedVisual::Sprite(_)
        | crate::core::visual::ResolvedVisual::FrameAnimation(_) => {
            let texture_handle = asset_server.load(asset_path.to_owned());
            let mut entity_id = None;
            spawn_standalone_static_sprite(
                parent,
                sprite_def,
                view_element,
                texture_handle,
                transform,
                node_name,
                &mut entity_id,
                asset_path,
            );
            entity_id.expect("spawn_standalone_static_sprite must set entity_id")
        }
    }
}

fn spawn_material_sprite(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sprite_def: &crate::core::view::layout::view_schema::SpriteDef,
    view_element: &Option<crate::core::view::components::ViewElement>,
    visual_path: &str,
    transform: Transform,
    node_name: &str,
    repeat_ctx: Option<&super::super::parsing::RepeatContext>,
) -> Entity {
    use crate::core::view::components::ShaderMaterial;
    use crate::core::view::layout::serde_types::vec2_tuple_to_static;
    use crate::core::view::reconcile::ShaderMaterialPendingSetup;

    let processed_sprite_def = if let Some(ctx) = repeat_ctx {
        preprocess_sprite_def_for_repeat(sprite_def, ctx)
    } else {
        sprite_def.clone()
    };
    let material_def = processed_sprite_def
        .material
        .as_ref()
        .expect("material must exist in spawn_material_sprite");

    let mut final_transform = Transform::from_translation(transform.translation)
        .with_scale(transform.scale)
        .with_rotation(transform.rotation);

    if let Some(pivot) = &sprite_def.pivot {
        let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
        let shift_x = (0.5 - pivot_x) * transform.scale.x;
        let shift_y = (0.5 - pivot_y) * transform.scale.y;
        let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
        final_transform.translation += shift;
    }

    let shader_path = if material_def.shader.starts_with("mod://") {
        material_def.shader.replacen("mod://", "", 1)
    } else {
        material_def.shader.clone()
    };
    let shader_handle = asset_server.load(&shader_path);

    let texture_handle: Handle<Image> = if visual_path.starts_with("procedural://") {
        Handle::default()
    } else {
        asset_server.load(visual_path.to_owned())
    };

    let shader_material = ShaderMaterial::from_def(shader_handle.clone(), material_def);

    let mut entity_cmd = parent.spawn((
        final_transform,
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(node_name.to_owned()),
        RonDrivenView,
        shader_material,
        ShaderMaterialPendingSetup {
            texture: texture_handle,
        },
    ));
    if let Some(view_element) = view_element {
        entity_cmd.insert(view_element.clone());
    }

    let entity_id = entity_cmd.id();
    info!(
        "[UI Sprite] Spawned shader material sprite '{}' (Entity {:?})",
        node_name, entity_id
    );
    entity_id
}
