//! Transform and layout component helpers for spawned View nodes.
//!
//! 已生成 View 节点的变换与布局组件辅助逻辑。

use super::repeat::build_transform;
use crate::core::view::layout::placement::{self, ViewLayoutOrigin};
use crate::core::view::layout::*;
use crate::core::view::ron_view::parsing::{self, PlayerDataView};
use bevy::prelude::*;

pub(super) fn resolve_node_or_local_transform(
    node_def: &ViewNodeDef,
    local_transform: Option<&SerializableTransform>,
    player_data: &PlayerDataView<'_>,
    repeat_ctx: Option<&parsing::RepeatContext>,
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

pub(super) fn combine_transforms(parent: Transform, child: Transform) -> Transform {
    Transform {
        translation: parent.translation + child.translation,
        rotation: parent.rotation * child.rotation,
        scale: parent.scale * child.scale,
    }
}

pub(super) fn combine_layout_transform(
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

pub(super) fn insert_layout_slot_components(
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

pub(super) fn node_display_is_none(node_def: &ViewNodeDef) -> bool {
    matches!(node_def.style.display, Some(SerializableDisplay::None))
}

pub(super) fn is_dynamic_transform(transform: &SerializableTransform) -> bool {
    transform.translation.as_ref().is_some_and(is_dynamic_vec3)
        || transform.scale.as_ref().is_some_and(is_dynamic_vec3)
        || transform
            .rotation
            .as_ref()
            .is_some_and(crate::core::sequencer::chapter_schema::Value::is_expr)
}

pub(super) fn transform_depends_on_time(transform: &SerializableTransform) -> bool {
    transform
        .translation
        .as_ref()
        .is_some_and(parsing::vec3_tuple_depends_on_time)
        || transform
            .scale
            .as_ref()
            .is_some_and(parsing::vec3_tuple_depends_on_time)
        || transform
            .rotation
            .as_ref()
            .is_some_and(parsing::expression_depends_on_time)
}
