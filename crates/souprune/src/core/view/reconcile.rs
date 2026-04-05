//! # reconcile module
//!
//! # 协调模块
//!
//! ## Module Overview / 模块概述
//!
//! Implements the declarative reconciliation architecture for the View system.
//! This module provides a single code path for both initial spawning and hot reloading.
//!
//! 实现 View 系统的声明式协调架构。
//! 该模块为初始生成和热重载提供统一的代码路径。
//!
//! ## Architecture / 架构
//!
//! ```text
//! ViewLayoutAsset + FactDatabase
//!              │
//!              ▼
//!   compute_desired_state() (pure function)
//!              │
//!              ▼
//!        DesiredViewTree
//!              │
//!              ▼
//!       reconcile() (diff algorithm)
//!              │
//!              ▼
//!        Vec<ViewDelta>
//!              │
//!              ▼
//!       apply_deltas() (ECS mutations)
//!              │
//!              ▼
//!          ECS World
//! ```

mod bindings;
mod compute;
mod delta;
mod diff;
mod resolve;
mod spawn_helpers;
mod spawn_shader;
mod system;
mod tree;

// Re-export public API for external use
// These may show as unused until external code adopts the reconciliation system
// 导出公共 API 供外部使用
// 在外部代码采用协调系统之前，这些可能显示为未使用
#[allow(unused_imports)]
pub use bindings::PropertyBinding;
#[allow(unused_imports)]
pub use compute::{ResolveContext, compute_desired_state};
#[allow(unused_imports)]
pub use delta::{DeltaStats, ViewDelta, apply_deltas};
#[allow(unused_imports)]
pub use diff::{build_current_tree, reconcile};
#[allow(unused_imports)]
pub use resolve::*;
// ViewReconciliationPlugin is used by CoreViewPlugin
// ViewReconciliationPlugin 被 CoreViewPlugin 使用
#[allow(unused_imports)]
pub use spawn_helpers::{
    SpawnContext, ViewElementSpec, build_text_config, spawn_sprite_entity, spawn_text_entity,
    spawn_viewbox_entity,
};
#[allow(unused_imports)]
pub use spawn_shader::{ShaderMaterialPendingSetup, spawn_shader_material_entity};
pub use system::ViewReconciliationPlugin;
#[allow(unused_imports)]
pub use system::{
    PendingReconciliations, ReconciliationEnabled, detect_asset_changes_system,
    detect_fact_changes_system, view_reconciliation_system,
};
#[allow(unused_imports)]
pub use tree::{
    CurrentElement, CurrentSprite, CurrentViewTree, DesiredElement, DesiredViewTree, ViewElementKey,
};
