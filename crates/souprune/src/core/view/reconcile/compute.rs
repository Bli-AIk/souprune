//! # compute.rs
//!
//! # 期望状态计算模块
//!
//! Pure function for computing desired view state from asset and facts.
//!
//! 从资源和事实计算期望视图状态的纯函数。

use super::resolve::{
    process_visible_when_for_repeat, resolve_material, resolve_node_transform, resolve_sprite,
    resolve_texts, resolve_viewbox_transform, resolve_visibility,
};
use super::tree::{DesiredElement, DesiredViewTree, ViewElementKey};
use crate::core::view::layout::{ViewLayoutAsset, ViewNodeDef};
use crate::core::view::ron_view::parsing::{
    DataPathResolvers, ExprFunctionResolvers, PlayerDataView, RepeatContext,
};
use bevy_fact_rule_event::{FactDatabase, LayeredFactDatabase};

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

    /// Create a new resolve context with local facts.
    /// 使用局部事实创建新的解析上下文。
    pub fn with_local_facts(
        layered_db: &'a LayeredFactDatabase,
        local_facts: &'a FactDatabase,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            player_data: PlayerDataView::with_local_facts(layered_db, local_facts),
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
/// * `local_facts` - Local facts for this view
/// * `namespace` - Namespace prefix for element names
///
/// # Returns
///
/// The desired view tree representing what the view should look like.
pub fn compute_desired_state(
    asset: &ViewLayoutAsset,
    global_facts: &LayeredFactDatabase,
    local_facts: &FactDatabase,
    namespace: &str,
    data_resolvers: Option<&DataPathResolvers>,
    expr_func_resolvers: Option<&ExprFunctionResolvers>,
) -> DesiredViewTree {
    let ctx = ResolveContext::with_local_facts(global_facts, local_facts, namespace)
        .with_data_resolvers(data_resolvers)
        .with_expr_functions(expr_func_resolvers);

    let roots = asset
        .roots
        .iter()
        .flat_map(|node_def| compute_element(&ctx, node_def, None))
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
) -> Vec<DesiredElement> {
    // Handle repeat expansion
    if let Some(repeat_spec) = &node_def.repeat {
        return expand_repeat(ctx, node_def, repeat_spec);
    }

    // Build element key
    let key = build_element_key(ctx, node_def, repeat_ctx);

    // Resolve transform: node transform wins, ViewBox uses offset, sprites use sprite.transform.
    let transform = if node_def.transform.is_some() {
        resolve_node_transform(&ctx.player_data, node_def, repeat_ctx)
    } else if let Some(ref vb) = node_def.view_box {
        resolve_viewbox_transform(vb, &ctx.player_data)
    } else {
        resolve_node_transform(&ctx.player_data, node_def, repeat_ctx)
    };

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
        .flat_map(|child| compute_element(ctx, child, repeat_ctx))
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

        // Resolve transform: node transform wins, ViewBox uses offset, sprites use sprite.transform.
        let transform = if node_def.transform.is_some() {
            resolve_node_transform(&ctx.player_data, node_def, Some(&repeat_ctx))
        } else if let Some(ref vb) = node_def.view_box {
            resolve_viewbox_transform(vb, &ctx.player_data)
        } else {
            resolve_node_transform(&ctx.player_data, node_def, Some(&repeat_ctx))
        };

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
            .flat_map(|child| compute_element(ctx, child, Some(&repeat_ctx)))
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
