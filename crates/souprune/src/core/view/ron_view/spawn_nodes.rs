//! # spawn_nodes.rs
//!
//! # spawn_nodes.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Acts as the main node-construction entry for the RON-driven View runtime. It inspects a
//! `ViewNodeDef`, chooses the correct spawning path for boxes, sprites, state sprites, and repeat
//! expansions, then delegates the detailed post-processing work to the local helper modules.
//!
//! RON 驱动 View 运行时的节点构建入口。它会检查 `ViewNodeDef`，为盒子、
//! 精灵、状态精灵以及 repeat 展开选择合适的生成路径，再把更细的后处理工作分发给本目录下
//! 的辅助子模块。

mod postprocess;
mod repeat;
mod sprite;

use super::super::components::*;
use super::super::layout::*;
use super::parsing::PlayerDataView;
use super::resources::RonDrivenView;
use super::spawn_helpers::{build_text_config, spawn_container_texts, spawn_ui_sprite};
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;

use postprocess::{apply_dynamic_element, apply_visible_when};
use repeat::{build_transform, resolve_repeat_item};
use sprite::spawn_standalone_sprite_node;

/// Spawn a single view node and its children.
///
/// 生成单个视图节点及其子节点。
pub fn spawn_view_node(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent_entity: Entity,
    node_def: &ViewNodeDef,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    namespace: &str,
) {
    if let Some(repeat) = &node_def.repeat {
        let array_len = if let Some(list) = player_data.get_fact_string_list(&repeat.source) {
            list.len()
        } else if let Some(list) = player_data.get_fact_int_list(&repeat.source) {
            list.len()
        } else {
            warn!(
                "[spawn_view_node] Repeat source '{}' not found for node '{}'",
                repeat.source, node_def.name
            );
            0
        };

        let limit = repeat.limit.unwrap_or(usize::MAX);
        let count = array_len.min(limit);

        info!(
            "[spawn_view_node] Repeating node '{}' {} times (source: '{}', len: {}, limit: {:?})",
            node_def.name, count, repeat.source, array_len, repeat.limit
        );

        for i in 0..count {
            let mut ctx = super::parsing::RepeatContext::new(i);

            if let Some(item_var) = &repeat.item_var
                && let Some(value) = resolve_repeat_item(player_data, &repeat.source, i)
            {
                ctx = ctx.with_item(item_var, value);
            }

            spawn_view_node_with_repeat_context(
                commands,
                asset_server,
                parent_entity,
                node_def,
                sprite_params,
                animation_assets,
                mortar_strings,
                player_data,
                namespace,
                Some(&ctx),
            );
        }
        return;
    }

    spawn_view_node_with_repeat_context(
        commands,
        asset_server,
        parent_entity,
        node_def,
        sprite_params,
        animation_assets,
        mortar_strings,
        player_data,
        namespace,
        None,
    );
}

fn spawn_view_node_with_repeat_context(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent_entity: Entity,
    node_def: &ViewNodeDef,
    sprite_params: &mut SpriteParams,
    animation_assets: &Assets<crate::core::character_asset::AnimationConfigAsset>,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
    player_data: &PlayerDataView<'_>,
    namespace: &str,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
) {
    let has_ui_box = node_def.view_box.is_some();
    let is_standalone_sprite = !has_ui_box && node_def.sprite.is_some();
    let is_state_sprite = !has_ui_box && node_def.state_sprite.is_some();
    let is_pure_container = !has_ui_box
        && !is_standalone_sprite
        && !is_state_sprite
        && (!node_def.texts.is_empty() || !node_def.children.is_empty());

    let node_name = if let Some(ctx) = repeat_ctx {
        if !node_def.name.is_empty() {
            format!("{}_{}", node_def.name, ctx.index)
        } else {
            String::new()
        }
    } else {
        node_def.name.clone()
    };

    let view_element = if !node_name.is_empty() {
        Some(crate::core::view::components::ViewElement::new(
            namespace.to_string(),
            node_name.clone(),
            node_def.tags.clone(),
        ))
    } else {
        None
    };

    let mut spawned_entity_id: Option<Entity> = None;

    commands.entity(parent_entity).with_children(|parent| {
        if is_state_sprite {
            let state_sprite_config = node_def
                .state_sprite
                .as_ref()
                .expect("state_sprite must exist when is_state_sprite is true");
            let transform = state_sprite_config
                .transform
                .as_ref()
                .map(|t| build_transform(t, player_data, None))
                .unwrap_or_default();

            info!(
                "[State Sprite] Spawning state sprite '{}' at position: {:?}",
                node_def.name, transform.translation
            );

            let state_sprite_state = StateSpriteState::from_config(state_sprite_config);
            let texture_handle: Handle<Image> = asset_server.load(&state_sprite_config.default);

            let mut entity_cmd = parent.spawn((
                Sprite {
                    image: texture_handle,
                    ..Default::default()
                },
                transform,
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new(node_def.name.clone()),
                RonDrivenView,
                state_sprite_state,
            ));

            if let Some(ref view_element) = view_element {
                entity_cmd.insert(view_element.clone());
            }

            let entity_id = entity_cmd.id();
            spawned_entity_id = Some(entity_id);

            info!(
                "[State Sprite] Spawned state sprite '{}' (Entity {:?})",
                node_def.name, entity_id
            );
            return;
        }

        if is_standalone_sprite {
            let sprite_def = node_def
                .sprite
                .as_ref()
                .expect("sprite must exist when is_standalone_sprite is true");
            let transform = sprite_def
                .transform
                .as_ref()
                .map(|t| build_transform(t, player_data, repeat_ctx))
                .unwrap_or_default();

            info!(
                "[UI Sprite] Spawning standalone sprite '{}' at position: {:?}, scale: {:?}",
                node_name, transform.translation, transform.scale
            );

            let visual_path = sprite_def.visual.path().to_owned();

            spawned_entity_id = Some(spawn_standalone_sprite_node(
                parent,
                asset_server,
                sprite_def,
                &view_element,
                &visual_path,
                transform,
                &node_def.name,
                repeat_ctx,
            ));
            return;
        }

        if has_ui_box {
            let view_box = node_def
                .view_box
                .as_ref()
                .expect("view_box must exist when has_ui_box is true");
            info!(
                "[UI Box] Creating ViewBox '{}' with dimensions: {}x{}, border: {}, offset: {:?}",
                node_def.name,
                view_box.width,
                view_box.height,
                view_box.border_width,
                view_box.offset
            );

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    build_text_config(text_def, mortar_strings, player_data)
                })
                .collect::<Vec<_>>();

            let offset = serializable_vec3_to_static(&view_box.offset);
            let is_dynamic_offset = is_dynamic_vec3(&view_box.offset);
            let fill_color = view_box
                .fill_color
                .as_ref()
                .map(|color| {
                    let (r, g, b, a) = color_tuple_to_static(color);
                    Color::srgba(r, g, b, a)
                })
                .unwrap_or(Color::BLACK);

            let runtime_view_box = ViewBox::new_full(
                view_box.width,
                view_box.height,
                view_box.border_width,
                texts,
                view_box.fill_shader.clone(),
                view_box.structure_file.clone(),
                fill_color,
            );
            let mut box_entity = parent.spawn((
                runtime_view_box,
                Transform::from_translation(offset),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new(node_def.name.clone()),
                RonDrivenView,
            ));

            if let Some(ref view_element) = view_element {
                box_entity.insert(view_element.clone());
            }

            if is_dynamic_offset {
                let dynamic_elem = DynamicViewElement {
                    sprite_def: None,
                    text_def: None,
                    view_box_def: Some(view_box.clone()),
                };
                box_entity.insert(dynamic_elem);
            }
            let needs_time_transform =
                is_dynamic_offset && super::parsing::vec3_tuple_depends_on_time(&view_box.offset);
            if needs_time_transform {
                box_entity.insert(TimeDependentTransform);
            }

            if !node_def.tags.is_empty() {
                box_entity.insert(super::super::components::ViewNodeTags(
                    node_def.tags.clone(),
                ));
            }

            info!(
                "[UI Box] Spawned ViewBox '{}' at offset: {:?} with structure_file: {:?}",
                node_def.name, offset, view_box.structure_file
            );

            if let Some(sprite_def) = &node_def.sprite {
                info!(
                    "[UI Box] Adding child sprite to ViewBox '{}': {:?}",
                    node_def.name,
                    sprite_def.visual.path()
                );
                spawn_ui_sprite(
                    &mut box_entity,
                    asset_server,
                    sprite_def,
                    sprite_params,
                    node_def.name.as_str(),
                    animation_assets,
                    player_data,
                );
            }

            spawned_entity_id = Some(box_entity.id());
            return;
        }

        if is_pure_container {
            info!(
                "[UI Container] Creating pure container '{}' with {} texts and {} children",
                node_def.name,
                node_def.texts.len(),
                node_def.children.len()
            );

            let mut container_entity = parent.spawn((
                ViewContainer,
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new(node_def.name.clone()),
                RonDrivenView,
            ));
            if let Some(ref view_element) = view_element {
                container_entity.insert(view_element.clone());
            }

            container_entity.with_children(|container_parent| {
                spawn_container_texts(
                    container_parent,
                    &node_def.texts,
                    mortar_strings,
                    player_data,
                );
            });

            spawned_entity_id = Some(container_entity.id());
        }
    });

    let Some(entity_id) = spawned_entity_id else {
        return;
    };

    if let Some(visible_when_expr) = &node_def.visible_when {
        apply_visible_when(
            commands,
            entity_id,
            visible_when_expr,
            &node_def.name,
            player_data,
            repeat_ctx,
        );
    }

    if is_standalone_sprite {
        apply_dynamic_element(commands, entity_id, node_def, repeat_ctx);
    }

    for child_def in &node_def.children {
        spawn_view_node(
            commands,
            asset_server,
            entity_id,
            child_def,
            sprite_params,
            animation_assets,
            mortar_strings,
            player_data,
            namespace,
        );
    }
}
