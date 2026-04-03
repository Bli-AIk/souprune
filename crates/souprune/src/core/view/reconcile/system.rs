//! # system.rs
//!
//! # 协调系统模块
//!
//! The unified view reconciliation system that replaces both spawn and hot reload.
//!
//! 替代 spawn 和热重载的统一视图协调系统。

use super::compute::compute_desired_state;
use super::delta::{DeltaStats, apply_deltas};
use super::diff::reconcile;
use super::tree::CurrentViewTree;
use crate::core::view::components::{ViewElement, ViewRoot};
use crate::core::view::layout::ViewLayoutAsset;
use bevy::asset::AssetEvent;
use bevy::ecs::prelude::MessageReader;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use std::collections::HashSet;

/// Marker component for view roots that use the reconciliation system.
/// 使用协调系统的视图根的标记组件。
#[derive(Component)]
pub struct ReconciliationEnabled;

/// Resource to track which assets need reconciliation this frame.
/// 资源，用于跟踪本帧需要协调的资产。
#[derive(Resource, Default)]
pub struct PendingReconciliations {
    pub asset_ids: HashSet<bevy::asset::AssetId<ViewLayoutAsset>>,
    pub force_all: bool,
}

impl PendingReconciliations {
    pub fn mark_asset(&mut self, id: bevy::asset::AssetId<ViewLayoutAsset>) {
        self.asset_ids.insert(id);
    }

    pub fn force_reconcile_all(&mut self) {
        self.force_all = true;
    }

    pub fn clear(&mut self) {
        self.asset_ids.clear();
        self.force_all = false;
    }

    pub fn needs_reconcile(&self, id: &bevy::asset::AssetId<ViewLayoutAsset>) -> bool {
        self.force_all || self.asset_ids.contains(id)
    }
}

/// System to detect asset changes and mark them for reconciliation.
/// 检测资产变化并标记需要协调的系统。
pub fn detect_asset_changes_system(
    mut asset_events: MessageReader<AssetEvent<ViewLayoutAsset>>,
    mut pending: ResMut<PendingReconciliations>,
) {
    for event in asset_events.read() {
        if let AssetEvent::Modified { id } = event {
            pending.mark_asset(*id);
            debug!(
                "[Reconciliation] Asset {:?} modified, marking for reconciliation",
                id
            );
        }
    }
}

/// System to detect fact changes and mark views for reconciliation.
/// Only marks views with `ActiveView` — non-interactive views (e.g., pure animation)
/// don't reference facts and shouldn't be re-evaluated on fact changes.
///
/// 检测事实变化并标记视图需要协调的系统。
/// 仅标记有 `ActiveView` 的视图——纯动画等非交互视图不引用事实，
/// 不应在事实变化时被重新求值。
pub fn detect_fact_changes_system(
    layered_db: Res<LayeredFactDatabase>,
    mut pending: ResMut<PendingReconciliations>,
    active_views: Query<
        &crate::core::view::ron_view::HotReloadableViewRoot,
        (
            With<ReconciliationEnabled>,
            With<crate::core::view::components::ActiveView>,
        ),
    >,
) {
    if layered_db.is_changed() {
        for hot_reload_root in active_views.iter() {
            pending.mark_asset(hot_reload_root.layout_handle.id());
        }
        debug!(
            "[Reconciliation] LayeredFactDatabase changed, marking {} active views for reconciliation",
            active_views.iter().count()
        );
    }
}

/// The main reconciliation system.
/// Computes desired state and applies minimal changes to reach it.
///
/// 主协调系统。
/// 计算期望状态并应用最小更改以达到该状态。
pub fn view_reconciliation_system(
    mut commands: Commands,
    mut pending: ResMut<PendingReconciliations>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    layered_db: Res<LayeredFactDatabase>,
    view_root_query: Query<
        (
            Entity,
            &ViewRoot,
            &crate::core::view::ron_view::HotReloadableViewRoot,
        ),
        With<ReconciliationEnabled>,
    >,
    view_element_query: Query<(
        Entity,
        &ViewElement,
        &Transform,
        &Visibility,
        Option<&ChildOf>,
        Option<&Sprite>,
        Option<&crate::core::view::components::VisibleWhen>,
        Option<&crate::core::view::components::ShaderMaterial>,
    )>,
    children_query: Query<&Children>,
) {
    // Skip if nothing needs reconciliation
    if pending.asset_ids.is_empty() && !pending.force_all {
        return;
    }

    for (root_entity, view_root, hot_reload_root) in view_root_query.iter() {
        let asset_id = hot_reload_root.layout_handle.id();

        // Check if this view needs reconciliation
        if !pending.needs_reconcile(&asset_id) {
            continue;
        }

        let Some(asset) = view_layouts.get(&hot_reload_root.layout_handle) else {
            warn!(
                "[Reconciliation] Asset not loaded for root {:?}",
                root_entity
            );
            continue;
        };

        // Compute desired state
        let namespace = &view_root.namespace;
        let desired = compute_desired_state(asset, &layered_db, &view_root.local_facts, namespace);

        // Build current state from ECS
        let current =
            build_current_tree_from_query(root_entity, &view_element_query, &children_query);

        // Compute deltas
        let deltas = reconcile(&current, &desired);

        // Log statistics
        let stats = DeltaStats::from_deltas(&deltas);
        if stats.has_changes() {
            debug!(
                "[Reconciliation] View {:?}: {} spawns, {} despawns, {} transform updates, {} visibility updates",
                root_entity,
                stats.spawns,
                stats.despawns,
                stats.transform_updates,
                stats.visibility_updates
            );
        }

        // Apply deltas
        apply_deltas(&mut commands, &deltas);
    }

    // Clear pending reconciliations for next frame
    pending.clear();
}

/// Build current view tree from ECS queries for a specific root entity.
/// 从特定根实体的 ECS 查询构建当前视图树。
fn build_current_tree_from_query(
    root_entity: Entity,
    view_element_query: &Query<(
        Entity,
        &ViewElement,
        &Transform,
        &Visibility,
        Option<&ChildOf>,
        Option<&Sprite>,
        Option<&crate::core::view::components::VisibleWhen>,
        Option<&crate::core::view::components::ShaderMaterial>,
    )>,
    children_query: &Query<&Children>,
) -> CurrentViewTree {
    let mut elements = Vec::new();

    // Collect all descendants of the root
    collect_descendants(
        root_entity,
        view_element_query,
        children_query,
        &mut elements,
    );

    build_current_tree_with_components(root_entity, &elements)
}

/// Element data collected from ECS queries.
/// 从 ECS 查询收集的元素数据。
struct CollectedElement {
    entity: Entity,
    full_name: String,
    transform: Transform,
    visibility: Visibility,
    parent: Option<Entity>,
    sprite: Option<super::tree::CurrentSprite>,
    visible_when_expr: Option<String>,
    has_shader_material: bool,
}

/// Recursively collect all view element descendants.
/// 递归收集所有视图元素子孙。
fn collect_descendants(
    entity: Entity,
    view_element_query: &Query<(
        Entity,
        &ViewElement,
        &Transform,
        &Visibility,
        Option<&ChildOf>,
        Option<&Sprite>,
        Option<&crate::core::view::components::VisibleWhen>,
        Option<&crate::core::view::components::ShaderMaterial>,
    )>,
    children_query: &Query<&Children>,
    result: &mut Vec<CollectedElement>,
) {
    // Add this entity if it has ViewElement
    if let Ok((
        e,
        view_element,
        transform,
        visibility,
        child_of,
        sprite_opt,
        visible_when_opt,
        shader_material_opt,
    )) = view_element_query.get(entity)
    {
        let parent = child_of.map(|c| c.parent());

        // Extract sprite properties if present
        let sprite = sprite_opt.map(|s| super::tree::CurrentSprite {
            color: s.color,
            flip_x: s.flip_x,
            flip_y: s.flip_y,
        });

        // Extract visible_when expression if present
        let visible_when_expr = visible_when_opt.map(|vw| vw.expression.clone());

        result.push(CollectedElement {
            entity: e,
            full_name: view_element.full_name.clone(),
            transform: *transform,
            visibility: *visibility,
            parent,
            sprite,
            visible_when_expr,
            has_shader_material: shader_material_opt.is_some(),
        });
    }

    // Recurse into children
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            collect_descendants(child, view_element_query, children_query, result);
        }
    }
}

/// Build current view tree from collected element data.
/// 从收集的元素数据构建当前视图树。
fn build_current_tree_with_components(
    _root_entity: Entity,
    elements: &[CollectedElement],
) -> CurrentViewTree {
    use super::tree::{CurrentElement, CurrentViewTree, ViewElementKey};

    let mut tree = CurrentViewTree::new();

    for elem in elements {
        // Extract repeat index from name if present (e.g., "EnemyHpBar_0" -> Some(0))
        // 从名称中提取重复索引（如 "EnemyHpBar_0" -> Some(0)）
        let repeat_index = extract_repeat_index(&elem.full_name);

        let key = if let Some(idx) = repeat_index {
            ViewElementKey::with_repeat_index(&elem.full_name, idx)
        } else {
            ViewElementKey::new(elem.full_name.clone())
        };

        tree.insert(CurrentElement {
            entity: elem.entity,
            key,
            transform: elem.transform,
            visibility: elem.visibility,
            parent: elem.parent,
            sprite: elem.sprite.clone(),
            visible_when_expr: elem.visible_when_expr.clone(),
            has_shader_material: elem.has_shader_material,
        });
    }

    tree
}

/// Extract repeat index from element name.
/// e.g., "namespace::EnemyHpBar_2" -> Some(2)
///
/// 从元素名称提取重复索引。
fn extract_repeat_index(full_name: &str) -> Option<usize> {
    // Find the last underscore followed by a number
    if let Some(underscore_pos) = full_name.rfind('_') {
        let suffix = &full_name[underscore_pos + 1..];
        suffix.parse::<usize>().ok()
    } else {
        None
    }
}

/// Plugin to enable the reconciliation system.
/// 启用协调系统的插件。
pub struct ViewReconciliationPlugin;

impl Plugin for ViewReconciliationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingReconciliations>().add_systems(
            Update,
            (
                detect_asset_changes_system,
                detect_fact_changes_system,
                view_reconciliation_system,
            )
                .chain(),
        );
    }
}
