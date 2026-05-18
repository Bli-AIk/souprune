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
use crate::core::view::layout::{
    SerializableDisplay, ViewLayoutAsset, ViewLayoutSlot, ViewLayoutSlots, ViewNodeDef,
    compute_taffy_layout, layout_child_path, layout_root_path,
};
use crate::core::view::ron_view::parsing::{
    DataPathResolvers, ExprFunctionResolvers, PlayerDataView, RepeatContext,
};
use bevy::prelude::{Transform, Vec2, Vec3};
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
    let layout_slots = compute_taffy_layout(asset, layout_viewport_size).ok();

    let roots = asset
        .roots
        .iter()
        .enumerate()
        .flat_map(|(root_idx, node_def)| {
            let node_path = layout_root_path(root_idx, node_def);
            compute_element(&ctx, node_def, None, layout_slots.as_ref(), &node_path)
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
) -> Vec<DesiredElement> {
    if node_display_is_none(node_def) {
        return Vec::new();
    }

    // Handle repeat expansion
    if let Some(repeat_spec) = &node_def.repeat {
        return expand_repeat(ctx, node_def, repeat_spec, layout_slots, node_path);
    }

    // Build element key
    let key = build_element_key(ctx, node_def, repeat_ctx);

    let layout_slot = layout_slots.and_then(|slots| slots.get(node_path));
    let transform = combine_layout_transform(
        layout_slot,
        resolve_element_transform(&ctx.player_data, node_def, repeat_ctx),
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
            compute_element(ctx, child, repeat_ctx, layout_slots, &child_path)
        })
        .collect();

    let mut element = DesiredElement::new(key, &node_def.name);
    element.tags = node_def.tags.clone();
    element.transform = transform;
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
) -> Vec<DesiredElement> {
    // Get the source array length
    let count = ctx
        .player_data
        .get_array_length(&format!("${}", repeat_spec.source))
        .unwrap_or(0);

    if count == 0 {
        return Vec::new();
    }

    let _index_var = repeat_spec
        .index_var
        .clone()
        .unwrap_or_else(|| "i".to_string());

    let mut elements = Vec::with_capacity(count);

    for i in 0..count {
        let repeat_ctx = RepeatContext::new(i);

        // Build key for this repeat instance
        let full_name = format!("{}::{}_{}", ctx.namespace, node_def.name, i);
        let key = ViewElementKey::with_repeat_index(full_name, i);

        let layout_slot = layout_slots.and_then(|slots| slots.get(node_path));
        let transform = combine_layout_transform(
            layout_slot,
            resolve_element_transform(&ctx.player_data, node_def, Some(&repeat_ctx)),
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
                let child_path = layout_child_path(node_path, child_idx, child);
                compute_element(ctx, child, Some(&repeat_ctx), layout_slots, &child_path)
            })
            .collect();

        let mut element = DesiredElement::new(key, &node_def.name);
        element.tags = node_def.tags.clone();
        element.transform = transform;
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

fn combine_layout_transform(slot: Option<&ViewLayoutSlot>, transform: Transform) -> Transform {
    let Some(slot) = slot else {
        return transform;
    };
    combine_transforms(
        Transform::from_translation(Vec3::new(slot.x, -slot.y, 0.0)),
        transform,
    )
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
mod tests {
    use super::*;
    use crate::core::sequencer::chapter_schema::Value;
    use crate::core::view::layout::{
        CoordinateSystem, SerializableJustifyContent, SerializableTransform, SerializableVal,
        StyleDef, UiFlexDirection,
    };
    use bevy_fact_rule_event::LayeredFactDatabase;

    fn asset(root: ViewNodeDef) -> ViewLayoutAsset {
        ViewLayoutAsset {
            roots: vec![root],
            requires: Vec::new(),
            facts: None,
            world_space: false,
            coordinate_system: CoordinateSystem::Standard,
            coordinate_space: None,
        }
    }

    fn node(name: &str, style: StyleDef, children: Vec<ViewNodeDef>) -> ViewNodeDef {
        ViewNodeDef {
            name: name.to_string(),
            tags: Vec::new(),
            style,
            transform: None,
            visible_when: None,
            background_color: None,
            border_color: None,
            image: None,
            sprite: None,
            state_sprite: None,
            texts: Vec::new(),
            view_box: None,
            children,
            repeat: None,
        }
    }

    #[test]
    fn desired_state_keeps_taffy_layout_offset() {
        let child = node(
            "Child",
            StyleDef {
                width: Some(SerializableVal::Px(100.0)),
                height: Some(SerializableVal::Px(40.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                justify_content: Some(SerializableJustifyContent::Center),
                ..Default::default()
            },
            vec![child],
        );
        let db = LayeredFactDatabase::new();
        let local = LocalState::new();

        let desired = compute_desired_state(
            &asset(root),
            Vec2::new(640.0, 480.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        assert_eq!(desired.roots[0].children[0].transform.translation.x, 270.0);
    }

    #[test]
    fn desired_state_skips_display_none_nodes() {
        let hidden = node(
            "Hidden",
            StyleDef {
                width: Some(SerializableVal::Px(100.0)),
                height: Some(SerializableVal::Px(40.0)),
                display: Some(SerializableDisplay::None),
                ..Default::default()
            },
            Vec::new(),
        );
        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![hidden],
        );
        let db = LayeredFactDatabase::new();
        let local = LocalState::new();

        let desired = compute_desired_state(
            &asset(root),
            Vec2::new(640.0, 480.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        assert!(desired.roots[0].children.is_empty());
    }

    #[test]
    fn desired_state_combines_layout_and_explicit_transform() {
        let mut child = node(
            "Child",
            StyleDef {
                width: Some(SerializableVal::Px(100.0)),
                height: Some(SerializableVal::Px(40.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        child.transform = Some(SerializableTransform {
            translation: Some((Value::Static(5.0), Value::Static(-6.0), Value::Static(7.0))),
            rotation: None,
            scale: None,
        });
        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                justify_content: Some(SerializableJustifyContent::Center),
                ..Default::default()
            },
            vec![child],
        );
        let db = LayeredFactDatabase::new();
        let local = LocalState::new();

        let desired = compute_desired_state(
            &asset(root),
            Vec2::new(640.0, 480.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        assert_eq!(
            desired.roots[0].children[0].transform.translation,
            Vec3::new(275.0, -6.0, 7.0)
        );
    }
}
