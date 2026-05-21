//! # compute.rs
//!
//! # 期望状态计算模块
//!
//! Pure function for computing desired view state from asset and facts.
//!
//! 从资源和事实计算期望视图状态的纯函数。

use super::resolve::{
    process_visible_when_for_repeat, resolve_material, resolve_node_transform, resolve_sprite,
    resolve_texts, resolve_transform, resolve_viewbox_transform, resolve_visibility,
};
use super::tree::{DesiredElement, DesiredViewTree, ViewElementKey};
use crate::core::view::LocalState;
use crate::core::view::layout::placement::{self, ViewLayoutOrigin};
use crate::core::view::layout::{
    SerializableDisplay, ViewLayoutAsset, ViewLayoutContext, ViewLayoutRect,
    ViewLayoutRepeatContext, ViewLayoutSlot, ViewLayoutSlots, ViewNodeDef, ViewSpaceDef,
    ViewWorld3dPlaneDef, apply_layout_repeat_context_to_text, compute_taffy_layout_with_context,
    layout_child_path, layout_repeat_path, layout_root_path,
};
use crate::core::view::ron_view::parsing::{
    DataPathResolvers, ExprFunctionResolvers, PlayerDataView, RepeatContext, resolve_text_content,
};
use bevy::prelude::{Transform, Vec2};
use bevy_fact_rule_event::LayeredFactDatabase;

/// Context for resolving expressions during desired state computation.
/// 计算期望状态时解析表达式的上下文。
pub struct ResolveContext<'a> {
    /// Player data view for expression evaluation
    /// 用于表达式求值的 PlayerDataView
    pub player_data: PlayerDataView<'a>,

    /// Namespace for element naming
    /// 元素命名的命名空间
    pub namespace: String,
}

impl<'a> ResolveContext<'a> {
    /// Create a new resolve context from global facts.
    /// 从全局事实创建新的解析上下文。
    pub fn new(layered_db: &'a LayeredFactDatabase, namespace: impl Into<String>) -> Self {
        Self {
            player_data: PlayerDataView::new(layered_db),
            namespace: namespace.into(),
        }
    }

    /// Create a new resolve context with read-only local state.
    /// 使用只读局部状态创建新的解析上下文。
    pub fn with_local_state(
        layered_db: &'a LayeredFactDatabase,
        local_state: &'a LocalState,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            player_data: PlayerDataView::with_local_state(layered_db, local_state),
            namespace: namespace.into(),
        }
    }

    /// Attach data path resolvers.
    pub fn with_data_resolvers(mut self, resolvers: Option<&'a DataPathResolvers>) -> Self {
        if let Some(r) = resolvers {
            self.player_data.set_data_path_resolvers(r);
        }
        self
    }

    /// Attach expression function resolvers.
    pub fn with_expr_functions(mut self, resolvers: Option<&'a ExprFunctionResolvers>) -> Self {
        self.player_data = self.player_data.with_expr_functions(resolvers);
        self
    }
}

/// Compute the desired view state from asset and facts.
/// This is a pure function with no side effects.
///
/// 从资源和事实计算期望的视图状态。
/// 这是一个无副作用的纯函数。
///
/// # Arguments
///
/// * `asset` - The view layout asset
/// * `global_facts` - Global fact database
/// * `local_state` - Read-only local state for this view
/// * `namespace` - Namespace prefix for element names
///
/// # Returns
///
/// The desired view tree representing what the view should look like.
pub fn compute_desired_state(
    asset: &ViewLayoutAsset,
    layout_viewport_size: Vec2,
    global_facts: &LayeredFactDatabase,
    local_state: &LocalState,
    namespace: &str,
    data_resolvers: Option<&DataPathResolvers>,
    expr_func_resolvers: Option<&ExprFunctionResolvers>,
) -> DesiredViewTree {
    let ctx = ResolveContext::with_local_state(global_facts, local_state, namespace)
        .with_data_resolvers(data_resolvers)
        .with_expr_functions(expr_func_resolvers);
    let mortar_strings = crate::extra::mortar::MortarStringTable::default();
    let repeat_count = |repeat: &crate::core::view::layout::RepeatDef| {
        Some(resolve_repeat_count(&ctx.player_data, repeat))
    };
    let repeat_item = |repeat: &crate::core::view::layout::RepeatDef, index: usize| {
        resolve_repeat_item(&ctx.player_data, &repeat.source, index)
    };
    let text_content = |content: &str, repeat_ctx: Option<&ViewLayoutRepeatContext>| {
        let content = apply_layout_repeat_context_to_text(content, repeat_ctx);
        resolve_text_content(&content, &mortar_strings, &ctx.player_data)
    };
    let layout_slots = compute_taffy_layout_with_context(
        asset,
        layout_viewport_size,
        ViewLayoutContext {
            repeat_count: &repeat_count,
            repeat_item: &repeat_item,
            text_content: &text_content,
        },
    )
    .ok();

    let spatial_plane = spatial_plane_for_asset(asset);
    let roots = asset
        .roots
        .iter()
        .enumerate()
        .flat_map(|(root_idx, node_def)| {
            let node_path = layout_root_path(root_idx, node_def);
            compute_element(
                &ctx,
                node_def,
                None,
                layout_slots.as_ref(),
                &node_path,
                None,
                ViewLayoutOrigin::TopLeft,
                spatial_plane,
            )
        })
        .collect();

    DesiredViewTree { roots }
}

/// Compute a single element and its children.
/// Handles repeat expansion.
///
/// 计算单个元素及其子元素。
/// 处理重复展开。
fn compute_element(
    ctx: &ResolveContext,
    node_def: &ViewNodeDef,
    repeat_ctx: Option<&RepeatContext>,
    layout_slots: Option<&ViewLayoutSlots>,
    node_path: &str,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    spatial_plane: Option<&ViewWorld3dPlaneDef>,
) -> Vec<DesiredElement> {
    if node_display_is_none(node_def) {
        return Vec::new();
    }

    // Handle repeat expansion
    if let Some(repeat_spec) = &node_def.repeat {
        return expand_repeat(
            ctx,
            node_def,
            repeat_spec,
            layout_slots,
            node_path,
            parent_slot,
            parent_origin,
            spatial_plane,
        );
    }

    // Build element key
    let key = build_element_key(ctx, node_def, repeat_ctx);

    let layout_slot = layout_slots.and_then(|slots| slots.get(node_path));
    let transform = combine_layout_transform(
        layout_slot,
        parent_slot,
        parent_origin,
        resolve_element_transform(&ctx.player_data, node_def, repeat_ctx),
        spatial_plane,
    );

    let visibility = resolve_visibility(
        &ctx.player_data,
        node_def.visible_when.as_deref(),
        repeat_ctx,
    );

    let sprite = resolve_sprite(node_def.sprite.as_ref());

    let texts = resolve_texts(&ctx.player_data, &node_def.texts, repeat_ctx);

    let material = resolve_material(node_def.sprite.as_ref());

    // Process visible_when expression for storage
    let visible_when_expr =
        process_visible_when_for_repeat(node_def.visible_when.as_deref(), repeat_ctx);

    // Recursively compute children
    let children = node_def
        .children
        .iter()
        .enumerate()
        .flat_map(|(child_idx, child)| {
            let child_path = layout_child_path(node_path, child_idx, child);
            compute_element(
                ctx,
                child,
                repeat_ctx,
                layout_slots,
                &child_path,
                layout_slot,
                child_parent_origin(node_def),
                spatial_plane,
            )
        })
        .collect();

    let mut element = DesiredElement::new(key, &node_def.name);
    element.tags = node_def.tags.clone();
    element.transform = transform;
    element.layout_rect = layout_slot.map(ViewLayoutRect::from);
    element.visibility = visibility;
    element.sprite = sprite;
    element.texts = texts;
    element.material = material;
    element.children = children;
    element.visible_when_expr = visible_when_expr;

    vec![element]
}

/// Expand a repeat node into multiple elements.
/// 将重复节点展开为多个元素。
fn expand_repeat(
    ctx: &ResolveContext,
    node_def: &ViewNodeDef,
    repeat_spec: &crate::core::view::layout::RepeatDef,
    layout_slots: Option<&ViewLayoutSlots>,
    node_path: &str,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    spatial_plane: Option<&ViewWorld3dPlaneDef>,
) -> Vec<DesiredElement> {
    let count = resolve_repeat_count(&ctx.player_data, repeat_spec);

    if count == 0 {
        return Vec::new();
    }

    let _index_var = repeat_spec
        .index_var
        .clone()
        .unwrap_or_else(|| "i".to_string());

    let mut elements = Vec::with_capacity(count);

    for i in 0..count {
        let mut repeat_ctx = RepeatContext::new(i);
        if let Some(index_var) = repeat_spec.index_var.as_deref()
            && !matches!(index_var, "i" | "index")
        {
            repeat_ctx = repeat_ctx.with_item(index_var, i.to_string());
        }

        if let Some(value) = resolve_repeat_item(&ctx.player_data, &repeat_spec.source, i) {
            let item_var = repeat_spec.item_var.as_deref().unwrap_or("item");
            repeat_ctx = repeat_ctx.with_item(item_var, value);
        }

        // Build key for this repeat instance
        let full_name = if ctx.namespace.is_empty() {
            format!("{}_{}", node_def.name, i)
        } else {
            format!("{}::{}_{}", ctx.namespace, node_def.name, i)
        };
        let key = ViewElementKey::with_repeat_index(full_name, i);

        let repeat_node_path = layout_repeat_path(node_path, i);
        let layout_slot = layout_slots.and_then(|slots| slots.get(&repeat_node_path));
        let transform = combine_layout_transform(
            layout_slot,
            parent_slot,
            parent_origin,
            resolve_element_transform(&ctx.player_data, node_def, Some(&repeat_ctx)),
            spatial_plane,
        );

        let visibility = resolve_visibility(
            &ctx.player_data,
            node_def.visible_when.as_deref(),
            Some(&repeat_ctx),
        );

        let sprite = resolve_sprite(node_def.sprite.as_ref());

        let texts = resolve_texts(&ctx.player_data, &node_def.texts, Some(&repeat_ctx));

        let material = resolve_material(node_def.sprite.as_ref());

        let visible_when_expr =
            process_visible_when_for_repeat(node_def.visible_when.as_deref(), Some(&repeat_ctx));

        // Children use the same repeat context
        let children = node_def
            .children
            .iter()
            .enumerate()
            .flat_map(|(child_idx, child)| {
                let child_path = layout_child_path(&repeat_node_path, child_idx, child);
                compute_element(
                    ctx,
                    child,
                    Some(&repeat_ctx),
                    layout_slots,
                    &child_path,
                    layout_slot,
                    child_parent_origin(node_def),
                    spatial_plane,
                )
            })
            .collect();

        let mut element = DesiredElement::new(key, &node_def.name);
        element.tags = node_def.tags.clone();
        element.transform = transform;
        element.layout_rect = layout_slot.map(ViewLayoutRect::from);
        element.visibility = visibility;
        element.sprite = sprite;
        element.texts = texts;
        element.material = material;
        element.children = children;
        element.visible_when_expr = visible_when_expr;

        elements.push(element);
    }

    elements
}

fn resolve_repeat_count(
    player_data: &PlayerDataView,
    repeat_spec: &crate::core::view::layout::RepeatDef,
) -> usize {
    let array_len = if let Some(list) = player_data.get_fact_string_list(&repeat_spec.source) {
        list.len()
    } else if let Some(list) = player_data.get_fact_int_list(&repeat_spec.source) {
        list.len()
    } else {
        0
    };

    array_len.min(repeat_spec.limit.unwrap_or(usize::MAX))
}

fn resolve_repeat_item(player_data: &PlayerDataView, source: &str, index: usize) -> Option<String> {
    if let Some(list) = player_data.get_fact_string_list(source) {
        list.into_iter().nth(index)
    } else if let Some(list) = player_data.get_fact_int_list(source) {
        list.into_iter().nth(index).map(|value| value.to_string())
    } else {
        None
    }
}

fn resolve_element_transform(
    player_data: &PlayerDataView,
    node_def: &ViewNodeDef,
    repeat_ctx: Option<&RepeatContext>,
) -> Transform {
    let local = if let Some(view_box) = &node_def.view_box {
        resolve_viewbox_transform(view_box, player_data)
    } else {
        resolve_transform(player_data, node_def.sprite.as_ref(), repeat_ctx)
    };

    if node_def.transform.is_some() {
        combine_transforms(
            resolve_node_transform(player_data, node_def, repeat_ctx),
            local,
        )
    } else {
        local
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

fn child_parent_origin(node_def: &ViewNodeDef) -> ViewLayoutOrigin {
    if node_is_pure_container(node_def) {
        ViewLayoutOrigin::TopLeft
    } else {
        ViewLayoutOrigin::Center
    }
}

fn node_is_pure_container(node_def: &ViewNodeDef) -> bool {
    node_def.view_box.is_none()
        && node_def.sprite.is_none()
        && node_def.state_sprite.is_none()
        && (!node_def.texts.is_empty() || !node_def.children.is_empty())
}

fn spatial_plane_for_asset(asset: &ViewLayoutAsset) -> Option<&ViewWorld3dPlaneDef> {
    let Some(ViewSpaceDef::World3dPlane(plane)) = asset.space.as_ref() else {
        return None;
    };
    Some(plane)
}

fn combine_transforms(parent: Transform, child: Transform) -> Transform {
    Transform {
        translation: parent.translation + child.translation,
        rotation: parent.rotation * child.rotation,
        scale: parent.scale * child.scale,
    }
}

fn node_display_is_none(node_def: &ViewNodeDef) -> bool {
    matches!(node_def.style.display, Some(SerializableDisplay::None))
}

/// Build the element key from context and node definition.
/// 从上下文和节点定义构建元素键。
fn build_element_key(
    ctx: &ResolveContext,
    node_def: &ViewNodeDef,
    repeat_ctx: Option<&RepeatContext>,
) -> ViewElementKey {
    let full_name = if ctx.namespace.is_empty() {
        node_def.name.clone()
    } else {
        format!("{}::{}", ctx.namespace, node_def.name)
    };

    if let Some(rctx) = repeat_ctx {
        let name_with_index = format!("{}_{}", full_name, rctx.index);
        ViewElementKey::with_repeat_index(name_with_index, rctx.index)
    } else {
        ViewElementKey::new(full_name)
    }
}

#[cfg(test)]
mod tests;
