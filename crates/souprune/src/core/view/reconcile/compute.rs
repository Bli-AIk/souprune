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
mod tests {
    use super::*;
    use crate::core::sequencer::chapter_schema::Value;
    use crate::core::view::layout::{
        CoordinateSystem, RepeatDef, SerializableJustifyContent, SerializableTransform,
        SerializableVal, StyleDef, UiFlexDirection, ViewBoxLogicDef, ViewCameraTargetDef,
        ViewSpaceDef, ViewWorld3dPlaneDef,
    };
    use bevy::prelude::Vec3;
    use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};

    fn asset(root: ViewNodeDef) -> ViewLayoutAsset {
        ViewLayoutAsset {
            roots: vec![root],
            requires: Vec::new(),
            facts: None,
            space: None,
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
            focus_policy: None,
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
        let rect = desired.roots[0].children[0]
            .layout_rect
            .expect("layout rect should be stored");
        assert_eq!(rect.x, 270.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 40.0);
    }

    #[test]
    fn desired_state_maps_spatial_layout_offset_to_plane_units() {
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
        let mut layout = asset(root);
        layout.space = Some(ViewSpaceDef::World3dPlane(Box::new(ViewWorld3dPlaneDef {
            transform: SerializableTransform::default(),
            rotation_degrees: None,
            plane_size: (6.4, 4.8),
            pixels_per_unit: 100.0,
            camera: ViewCameraTargetDef::Main,
            anchor: Default::default(),
            orientation: Default::default(),
            depth: Default::default(),
            input: Default::default(),
        })));
        let db = LayeredFactDatabase::new();
        let local = LocalState::new();

        let desired = compute_desired_state(
            &layout,
            Vec2::new(640.0, 480.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        assert_eq!(
            desired.roots[0].children[0].transform.translation,
            Vec3::new(2.7, 0.0, 0.0)
        );
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

    #[test]
    fn desired_state_treats_taffy_child_slots_as_parent_local() {
        let mut leaf = node(
            "Leaf",
            StyleDef {
                width: Some(SerializableVal::Px(40.0)),
                height: Some(SerializableVal::Px(20.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        leaf.view_box = Some(ViewBoxLogicDef {
            width: 40.0,
            height: 20.0,
            border_width: 0.0,
            offset: (
                Value::Static(20.0),
                Value::Static(-10.0),
                Value::Static(0.0),
            ),
            fill_shader: None,
            structure_file: None,
            fill_color: None,
        });

        let mut row = node(
            "Row",
            StyleDef {
                width: Some(SerializableVal::Px(160.0)),
                height: Some(SerializableVal::Px(80.0)),
                padding: Some(crate::core::view::layout::SerializableRect {
                    left: SerializableVal::Px(10.0),
                    right: SerializableVal::Px(0.0),
                    top: SerializableVal::Px(20.0),
                    bottom: SerializableVal::Px(0.0),
                }),
                ..Default::default()
            },
            vec![leaf],
        );
        row.view_box = Some(ViewBoxLogicDef {
            width: 160.0,
            height: 80.0,
            border_width: 0.0,
            offset: (
                Value::Static(80.0),
                Value::Static(-40.0),
                Value::Static(0.0),
            ),
            fill_shader: None,
            structure_file: None,
            fill_color: None,
        });

        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(240.0)),
                height: Some(SerializableVal::Px(120.0)),
                padding: Some(crate::core::view::layout::SerializableRect {
                    left: SerializableVal::Px(30.0),
                    right: SerializableVal::Px(0.0),
                    top: SerializableVal::Px(40.0),
                    bottom: SerializableVal::Px(0.0),
                }),
                ..Default::default()
            },
            vec![row],
        );

        let db = LayeredFactDatabase::new();
        let local = LocalState::new();

        let desired = compute_desired_state(
            &asset(root),
            Vec2::new(960.0, 540.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        let row = &desired.roots[0].children[0];
        let leaf = &row.children[0];

        assert_eq!(row.transform.translation, Vec3::new(110.0, -80.0, 0.0));
        assert_eq!(leaf.transform.translation, Vec3::new(-50.0, 10.0, 0.0));
    }

    #[test]
    fn desired_state_places_observer_demo_visuals_inside_surface() {
        let asset: ViewLayoutAsset = ron::from_str(include_str!(
            "../../../../examples/assets/view/layout_observer_demo.view.ron"
        ))
        .expect("observer example asset should parse");
        let db = LayeredFactDatabase::new();
        let local = LocalState::new();

        let desired =
            compute_desired_state(&asset, Vec2::new(960.0, 540.0), &db, &local, "", None, None);

        let surface = desired
            .roots
            .iter()
            .find(|element| element.name == "ObserverSurface")
            .expect("surface element");
        let surface_bounds = visual_bounds(surface, Vec3::ZERO).expect("surface bounds");

        for name in [
            "ObserverHeader",
            "ObserverBody",
            "ObserverBadge",
            "HeaderLeft",
            "ObserverLineA",
        ] {
            let bounds =
                find_descendant_bounds(surface, name, Vec3::ZERO).expect("descendant bounds");
            assert!(
                contains_bounds(surface_bounds, bounds),
                "{name} bounds {:?} should be inside surface {:?}",
                bounds,
                surface_bounds
            );
        }
    }

    #[test]
    fn desired_state_repeat_respects_limit_for_local_string_list() {
        let mut root = node("Item", StyleDef::default(), Vec::new());
        root.repeat = Some(RepeatDef {
            source: "names".to_string(),
            limit: Some(2),
            index_var: None,
            item_var: None,
        });
        let db = LayeredFactDatabase::new();
        let mut local = LocalState::new();
        local.set(
            "names",
            FactValue::StringList(vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
            ]),
        );

        let desired = compute_desired_state(
            &asset(root),
            Vec2::new(640.0, 480.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        assert_eq!(desired.roots.len(), 2);
        assert_eq!(desired.roots[0].key.full_name, "Item_0");
        assert_eq!(desired.roots[1].key.full_name, "Item_1");
    }

    #[test]
    fn desired_state_repeat_binds_int_item_var_for_transform() {
        let mut root = node("Value", StyleDef::default(), Vec::new());
        root.repeat = Some(RepeatDef {
            source: "values".to_string(),
            limit: None,
            index_var: None,
            item_var: Some("value".to_string()),
        });
        root.transform = Some(SerializableTransform {
            translation: Some((
                Value::Expr("@value".to_string()),
                Value::Static(0.0),
                Value::Static(0.0),
            )),
            rotation: None,
            scale: None,
        });
        let db = LayeredFactDatabase::new();
        let mut local = LocalState::new();
        local.set("values", FactValue::IntList(vec![4, 8]));

        let desired = compute_desired_state(
            &asset(root),
            Vec2::new(640.0, 480.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        let xs: Vec<f32> = desired
            .roots
            .iter()
            .map(|root| root.transform.translation.x)
            .collect();
        assert_eq!(xs, vec![4.0, 8.0]);
    }

    #[test]
    fn desired_state_repeat_count_updates_sibling_layout_rect() {
        let mut repeated = node(
            "Item",
            StyleDef {
                width: Some(SerializableVal::Px(50.0)),
                height: Some(SerializableVal::Px(20.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        repeated.repeat = Some(RepeatDef {
            source: "items".to_string(),
            limit: None,
            index_var: None,
            item_var: None,
        });
        let sibling = node(
            "Tail",
            StyleDef {
                width: Some(SerializableVal::Px(50.0)),
                height: Some(SerializableVal::Px(20.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(300.0)),
                height: Some(SerializableVal::Px(100.0)),
                flex_direction: Some(UiFlexDirection::Row),
                justify_content: Some(SerializableJustifyContent::Center),
                ..Default::default()
            },
            vec![repeated, sibling],
        );
        let db = LayeredFactDatabase::new();
        let mut local = LocalState::new();
        local.set("items", FactValue::StringList(vec!["one".to_string()]));
        let one_item = compute_desired_state(
            &asset(root.clone()),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        local.set(
            "items",
            FactValue::StringList(vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
            ]),
        );
        let three_items = compute_desired_state(
            &asset(root),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        let one_tail = one_item.roots[0].children.last().expect("tail child");
        let three_tail = three_items.roots[0].children.last().expect("tail child");
        assert_eq!(one_tail.name, "Tail");
        assert_eq!(three_tail.name, "Tail");
        assert_eq!(one_tail.layout_rect.expect("tail rect").x, 150.0);
        assert_eq!(three_tail.layout_rect.expect("tail rect").x, 200.0);
    }

    #[test]
    fn desired_state_fact_text_length_updates_fit_sibling_layout_rect() {
        let mut label = node(
            "Label",
            StyleDef {
                sizing: Some(crate::core::view::layout::ViewSizingDef::Fit),
                ..Default::default()
            },
            Vec::new(),
        );
        label.texts.push(crate::core::view::layout::TextDef {
            id: "label_text".to_string(),
            content: Some("{$label}".to_string()),
            font: "default".to_string(),
            align: None,
            anchor: None,
            world_scale: (Value::Static(1.0), Value::Static(1.0)),
            color: (
                Value::Static(1.0),
                Value::Static(1.0),
                Value::Static(1.0),
                Value::Static(1.0),
            ),
            transform: SerializableTransform::default(),
            line_height: None,
            char_spacing: None,
            word_spacing: None,
            text_style: None,
            conditional_style: None,
            visible_when: None,
        });
        let marker = node(
            "Marker",
            StyleDef {
                width: Some(SerializableVal::Px(20.0)),
                height: Some(SerializableVal::Px(20.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(300.0)),
                height: Some(SerializableVal::Px(100.0)),
                flex_direction: Some(UiFlexDirection::Row),
                ..Default::default()
            },
            vec![label, marker],
        );
        let db = LayeredFactDatabase::new();
        let mut local = LocalState::new();
        local.set("label", FactValue::String("A".to_string()));
        let short = compute_desired_state(
            &asset(root.clone()),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        local.set("label", FactValue::String("ABCDEFG".to_string()));
        let long = compute_desired_state(
            &asset(root),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        let short_marker = short.roots[0].children.last().expect("marker child");
        let long_marker = long.roots[0].children.last().expect("marker child");
        assert!(
            long_marker.layout_rect.expect("long marker rect").x
                > short_marker.layout_rect.expect("short marker rect").x
        );
    }

    #[test]
    fn desired_state_repeat_item_text_length_updates_fit_sibling_layout_rect() {
        let mut label = node(
            "Label",
            StyleDef {
                sizing: Some(crate::core::view::layout::ViewSizingDef::Fit),
                ..Default::default()
            },
            Vec::new(),
        );
        label.repeat = Some(RepeatDef {
            source: "items".to_string(),
            limit: Some(1),
            index_var: None,
            item_var: None,
        });
        label.texts.push(crate::core::view::layout::TextDef {
            id: "label_text".to_string(),
            content: Some("@item".to_string()),
            font: "default".to_string(),
            align: None,
            anchor: None,
            world_scale: (Value::Static(1.0), Value::Static(1.0)),
            color: (
                Value::Static(1.0),
                Value::Static(1.0),
                Value::Static(1.0),
                Value::Static(1.0),
            ),
            transform: SerializableTransform::default(),
            line_height: None,
            char_spacing: None,
            word_spacing: None,
            text_style: None,
            conditional_style: None,
            visible_when: None,
        });
        let marker = node(
            "Marker",
            StyleDef {
                width: Some(SerializableVal::Px(20.0)),
                height: Some(SerializableVal::Px(20.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        let root = node(
            "Root",
            StyleDef {
                width: Some(SerializableVal::Px(300.0)),
                height: Some(SerializableVal::Px(100.0)),
                flex_direction: Some(UiFlexDirection::Row),
                ..Default::default()
            },
            vec![label, marker],
        );
        let db = LayeredFactDatabase::new();
        let mut local = LocalState::new();
        local.set("items", FactValue::StringList(vec!["A".to_string()]));
        let short = compute_desired_state(
            &asset(root.clone()),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        local.set(
            "items",
            FactValue::StringList(vec!["LONG_LABEL".to_string()]),
        );
        let long = compute_desired_state(
            &asset(root),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        let short_marker = short.roots[0].children.last().expect("marker child");
        let long_marker = long.roots[0].children.last().expect("marker child");
        assert!(
            long_marker.layout_rect.expect("long marker rect").x
                > short_marker.layout_rect.expect("short marker rect").x
        );
    }

    #[test]
    fn desired_state_repeat_default_item_var_resolves_text_content() {
        let mut label = node("Label", StyleDef::default(), Vec::new());
        label.repeat = Some(RepeatDef {
            source: "items".to_string(),
            limit: Some(1),
            index_var: None,
            item_var: None,
        });
        label.texts.push(crate::core::view::layout::TextDef {
            id: "label_text".to_string(),
            content: Some("@item".to_string()),
            font: "default".to_string(),
            align: None,
            anchor: None,
            world_scale: (Value::Static(1.0), Value::Static(1.0)),
            color: (
                Value::Static(1.0),
                Value::Static(1.0),
                Value::Static(1.0),
                Value::Static(1.0),
            ),
            transform: SerializableTransform::default(),
            line_height: None,
            char_spacing: None,
            word_spacing: None,
            text_style: None,
            conditional_style: None,
            visible_when: None,
        });
        let db = LayeredFactDatabase::new();
        let mut local = LocalState::new();
        local.set("items", FactValue::StringList(vec!["Alpha".to_string()]));

        let desired = compute_desired_state(
            &asset(label),
            Vec2::new(300.0, 100.0),
            &db,
            &local,
            "",
            None,
            None,
        );

        assert_eq!(desired.roots[0].texts[0].content, "Alpha");
    }

    #[derive(Debug, Clone, Copy)]
    struct Bounds {
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
    }

    fn visual_bounds(element: &DesiredElement, parent_translation: Vec3) -> Option<Bounds> {
        let center = parent_translation + element.transform.translation;
        let rect = element.layout_rect?;
        Some(Bounds {
            left: center.x - rect.width * 0.5,
            right: center.x + rect.width * 0.5,
            top: center.y + rect.height * 0.5,
            bottom: center.y - rect.height * 0.5,
        })
    }

    fn contains_bounds(container: Bounds, child: Bounds) -> bool {
        child.left >= container.left
            && child.right <= container.right
            && child.top <= container.top
            && child.bottom >= container.bottom
    }

    fn find_descendant_bounds(
        root: &DesiredElement,
        name: &str,
        parent_translation: Vec3,
    ) -> Option<Bounds> {
        let translation = parent_translation + root.transform.translation;
        root.children.iter().find_map(|child| {
            if child.name == name {
                visual_bounds(child, translation)
            } else {
                find_descendant_bounds(child, name, translation)
            }
        })
    }
}
