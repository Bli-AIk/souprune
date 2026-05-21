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
use super::super::layout::placement::{self, ViewLayoutOrigin};
use super::super::layout::*;
use super::parsing::PlayerDataView;
use super::resources::RonDrivenView;
use super::spawn_helpers::{build_text_config, spawn_container_texts, spawn_ui_sprite};
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;

use postprocess::{apply_dynamic_element, apply_visible_when};
use repeat::{build_transform, build_vec3, resolve_repeat_item};
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
    layout_slots: Option<&ViewLayoutSlots>,
    node_path: &str,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    spatial_plane: Option<&ViewWorld3dPlaneDef>,
) {
    if node_display_is_none(node_def) {
        return;
    }

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

        debug!(
            "[spawn_view_node] Repeating node '{}' {} times (source: '{}', len: {}, limit: {:?})",
            node_def.name, count, repeat.source, array_len, repeat.limit
        );

        for i in 0..count {
            let mut ctx = super::parsing::RepeatContext::new(i);
            let repeat_node_path = layout_repeat_path(node_path, i);
            if let Some(index_var) = repeat.index_var.as_deref()
                && !matches!(index_var, "i" | "index")
            {
                ctx = ctx.with_item(index_var, i.to_string());
            }

            if let Some(value) = resolve_repeat_item(player_data, &repeat.source, i) {
                let item_var = repeat.item_var.as_deref().unwrap_or("item");
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
                layout_slots,
                &repeat_node_path,
                parent_slot,
                parent_origin,
                spatial_plane,
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
        layout_slots,
        node_path,
        parent_slot,
        parent_origin,
        spatial_plane,
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
    layout_slots: Option<&ViewLayoutSlots>,
    node_path: &str,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    spatial_plane: Option<&ViewWorld3dPlaneDef>,
) {
    let has_view_box = node_def.view_box.is_some();
    let is_standalone_sprite = !has_view_box && node_def.sprite.is_some();
    let is_state_sprite = !has_view_box && node_def.state_sprite.is_some();
    let is_pure_container = !has_view_box
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
    let layout_slot = layout_slots.and_then(|slots| slots.get(node_path));

    commands.entity(parent_entity).with_children(|parent| {
        if is_state_sprite {
            let state_sprite_config = node_def
                .state_sprite
                .as_ref()
                .expect("state_sprite must exist when is_state_sprite is true");
            let transform = resolve_node_or_local_transform(
                node_def,
                state_sprite_config.transform.as_ref(),
                player_data,
                repeat_ctx,
            );
            let transform = combine_layout_transform(
                layout_slot,
                parent_slot,
                parent_origin,
                transform,
                spatial_plane,
            );

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
            let transform = resolve_node_or_local_transform(
                node_def,
                sprite_def.transform.as_ref(),
                player_data,
                repeat_ctx,
            );
            let transform = combine_layout_transform(
                layout_slot,
                parent_slot,
                parent_origin,
                transform,
                spatial_plane,
            );

            info!(
                "[View Sprite] Spawning standalone sprite '{}' at position: {:?}, scale: {:?}",
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

        if has_view_box {
            let view_box = node_def
                .view_box
                .as_ref()
                .expect("view_box must exist when has_view_box is true");
            info!(
                "[View Box] Creating ViewBox '{}' with dimensions: {}x{}, border: {}, offset: {:?}",
                node_def.name,
                view_box.width,
                view_box.height,
                view_box.border_width,
                view_box.offset
            );

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| build_text_config(text_def, mortar_strings, player_data))
                .collect::<Vec<_>>();

            let offset = build_vec3(&view_box.offset, player_data, repeat_ctx);
            let transform = node_def
                .transform
                .as_ref()
                .map(|transform_def| {
                    combine_transforms(
                        build_transform(transform_def, player_data, repeat_ctx),
                        Transform::from_translation(offset),
                    )
                })
                .unwrap_or_else(|| Transform::from_translation(offset));
            let transform = combine_layout_transform(
                layout_slot,
                parent_slot,
                parent_origin,
                transform,
                spatial_plane,
            );
            let is_dynamic_node_transform = node_def
                .transform
                .as_ref()
                .is_some_and(is_dynamic_transform);
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
                transform,
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

            if is_dynamic_node_transform || is_dynamic_offset {
                let dynamic_elem = DynamicViewElement {
                    node_transform: node_def.transform.clone(),
                    sprite_def: None,
                    text_def: None,
                    view_box_def: Some(view_box.clone()),
                };
                box_entity.insert(dynamic_elem);
            }
            let needs_time_transform = node_def
                .transform
                .as_ref()
                .is_some_and(transform_depends_on_time)
                || (is_dynamic_offset
                    && super::parsing::vec3_tuple_depends_on_time(&view_box.offset));
            if needs_time_transform {
                box_entity.insert(TimeDependentTransform);
            }

            if !node_def.tags.is_empty() {
                box_entity.insert(super::super::components::ViewNodeTags(
                    node_def.tags.clone(),
                ));
            }

            info!(
                "[View Box] Spawned ViewBox '{}' at offset: {:?} with structure_file: {:?}",
                node_def.name, offset, view_box.structure_file
            );

            if let Some(sprite_def) = &node_def.sprite {
                info!(
                    "[View Box] Adding child sprite to ViewBox '{}': {:?}",
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
                "[View Container] Creating pure container '{}' with {} texts and {} children",
                node_def.name,
                node_def.texts.len(),
                node_def.children.len()
            );

            let mut container_entity = parent.spawn((
                ViewContainer,
                combine_layout_transform(
                    layout_slot,
                    parent_slot,
                    parent_origin,
                    node_def
                        .transform
                        .as_ref()
                        .map(|transform| build_transform(transform, player_data, repeat_ctx))
                        .unwrap_or_default(),
                    spatial_plane,
                ),
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

    insert_layout_slot_components(commands, entity_id, layout_slots, node_path, layout_slot);

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

    for (child_idx, child_def) in node_def.children.iter().enumerate() {
        let child_path = layout_child_path(node_path, child_idx, child_def);
        let child_parent_origin = if is_pure_container {
            ViewLayoutOrigin::TopLeft
        } else {
            ViewLayoutOrigin::Center
        };
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
            layout_slots,
            &child_path,
            layout_slot,
            child_parent_origin,
            spatial_plane,
        );
    }
}

fn resolve_node_or_local_transform(
    node_def: &ViewNodeDef,
    local_transform: Option<&SerializableTransform>,
    player_data: &PlayerDataView<'_>,
    repeat_ctx: Option<&super::parsing::RepeatContext>,
) -> Transform {
    let local =
        local_transform.map(|transform| build_transform(transform, player_data, repeat_ctx));
    let Some(node_transform) = &node_def.transform else {
        return local.unwrap_or_default();
    };

    let node = build_transform(node_transform, player_data, repeat_ctx);
    local
        .map(|local| combine_transforms(node, local))
        .unwrap_or(node)
}

fn combine_transforms(parent: Transform, child: Transform) -> Transform {
    Transform {
        translation: parent.translation + child.translation,
        rotation: parent.rotation * child.rotation,
        scale: parent.scale * child.scale,
    }
}

fn combine_layout_transform(
    slot: Option<&ViewLayoutSlot>,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    transform: Transform,
    spatial_plane: Option<&ViewWorld3dPlaneDef>,
) -> Transform {
    if let Some(plane) = spatial_plane {
        return placement::combine_spatial_layout_transform(
            slot,
            parent_slot,
            parent_origin,
            plane.pixels_per_unit,
            transform,
        );
    }
    placement::combine_layout_transform(slot, parent_slot, parent_origin, transform)
}

fn insert_layout_slot_components(
    commands: &mut Commands,
    entity_id: Entity,
    layout_slots: Option<&ViewLayoutSlots>,
    node_path: &str,
    layout_slot: Option<&ViewLayoutSlot>,
) {
    let mut entity_commands = commands.entity(entity_id);
    if let Some(slot) = layout_slot {
        entity_commands.try_insert(ViewLayoutRect::from(slot));
    }
    #[cfg(feature = "debug")]
    if let Some(debug_metadata) = layout_slots
        .and_then(|slots| slots.debug_metadata(node_path))
        .cloned()
    {
        entity_commands.try_insert(debug_metadata);
    }
    if let Some(clip_rect) = layout_slots
        .and_then(|slots| slots.clip_rect(node_path))
        .copied()
    {
        entity_commands.try_insert(clip_rect);
    }
    if let Some(scroll_state) = layout_slots
        .and_then(|slots| slots.scroll_state(node_path))
        .copied()
    {
        entity_commands.try_insert(scroll_state);
    }
}

fn node_display_is_none(node_def: &ViewNodeDef) -> bool {
    matches!(node_def.style.display, Some(SerializableDisplay::None))
}

fn is_dynamic_transform(transform: &SerializableTransform) -> bool {
    transform.translation.as_ref().is_some_and(is_dynamic_vec3)
        || transform.scale.as_ref().is_some_and(is_dynamic_vec3)
        || transform
            .rotation
            .as_ref()
            .is_some_and(crate::core::sequencer::chapter_schema::Value::is_expr)
}

fn transform_depends_on_time(transform: &SerializableTransform) -> bool {
    transform
        .translation
        .as_ref()
        .is_some_and(super::parsing::vec3_tuple_depends_on_time)
        || transform
            .scale
            .as_ref()
            .is_some_and(super::parsing::vec3_tuple_depends_on_time)
        || transform
            .rotation
            .as_ref()
            .is_some_and(super::parsing::expression_depends_on_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::ViewLayoutSlot;

    #[test]
    fn layout_slot_offsets_existing_transform_in_view_coordinates() {
        let slot = ViewLayoutSlot {
            path: "Root/Child".to_string(),
            name: "Child".to_string(),
            x: 210.0,
            y: 120.0,
            width: 100.0,
            height: 40.0,
        };
        let explicit = Transform::from_translation(Vec3::new(5.0, -6.0, 7.0));

        let combined =
            combine_layout_transform(Some(&slot), None, ViewLayoutOrigin::Center, explicit, None);

        assert_eq!(combined.translation, Vec3::new(215.0, -126.0, 7.0));
    }

    #[test]
    fn spatial_layout_slot_uses_plane_units_for_translation() {
        let slot = ViewLayoutSlot {
            path: "Root/Child".to_string(),
            name: "Child".to_string(),
            x: 210.0,
            y: 120.0,
            width: 100.0,
            height: 40.0,
        };
        let plane = ViewWorld3dPlaneDef {
            transform: SerializableTransform::default(),
            rotation_degrees: None,
            plane_size: (6.4, 4.8),
            pixels_per_unit: 100.0,
            camera: ViewCameraTargetDef::Main,
            anchor: Default::default(),
            orientation: Default::default(),
            depth: Default::default(),
            input: Default::default(),
        };
        let explicit = Transform::from_translation(Vec3::new(0.5, -0.25, 7.0));

        let combined = combine_layout_transform(
            Some(&slot),
            None,
            ViewLayoutOrigin::Center,
            explicit,
            Some(&plane),
        );

        assert_eq!(combined.translation, Vec3::new(2.6, -1.45, 7.0));
    }

    #[derive(Resource)]
    struct LayoutMetadataTarget(Entity);

    #[derive(Resource)]
    struct LayoutMetadataSlots(ViewLayoutSlots);

    fn insert_layout_metadata_for_test(
        mut commands: Commands,
        target: Res<LayoutMetadataTarget>,
        slots: Res<LayoutMetadataSlots>,
    ) {
        let slot = slots.0.get("Root/Child");
        insert_layout_slot_components(&mut commands, target.0, Some(&slots.0), "Root/Child", slot);
    }

    #[test]
    fn layout_slot_metadata_is_inserted_as_runtime_components() {
        let mut slots = ViewLayoutSlots::new();
        slots.push_with_metadata(
            ViewLayoutSlot {
                path: "Root/Child".to_string(),
                name: "Child".to_string(),
                x: 210.0,
                y: 120.0,
                width: 100.0,
                height: 40.0,
            },
            Some(ViewClipRect::new(210.0, 120.0, 100.0, 40.0)),
            Some(ViewScrollState::default()),
        );

        let mut app = App::new();
        let entity = app.world_mut().spawn_empty().id();
        app.insert_resource(LayoutMetadataTarget(entity));
        app.insert_resource(LayoutMetadataSlots(slots));
        app.add_systems(Update, insert_layout_metadata_for_test);

        app.update();

        let entity_ref = app.world().entity(entity);
        assert!(entity_ref.contains::<ViewLayoutRect>());
        assert!(entity_ref.contains::<ViewClipRect>());
        assert!(entity_ref.contains::<ViewScrollState>());
    }

    #[test]
    fn display_none_node_is_not_spawned() {
        let node = ViewNodeDef {
            name: "Hidden".to_string(),
            tags: Vec::new(),
            style: StyleDef {
                display: Some(SerializableDisplay::None),
                ..Default::default()
            },
            transform: None,
            focus_policy: None,
            visible_when: None,
            background_color: None,
            border_color: None,
            image: None,
            sprite: None,
            state_sprite: None,
            texts: Vec::new(),
            view_box: None,
            children: Vec::new(),
            repeat: None,
        };

        assert!(node_display_is_none(&node));
    }
}
