//! Defines the marker components and reload-tracking resources used by RON-driven views.
//!
//! 定义 RON 驱动 View 所需的标记组件与重载跟踪资源。
//!
//! Holds the lightweight state that other view runtime modules share:
//! which entities belong to generated views, which roots can be hot reloaded,
//! and which layout assets are waiting to be rebuilt. These types do not do the
//! work themselves, but they are the glue that lets reload/setup/update systems
//! talk about the same set of view entities.
//!
//! 保存的是其他 View 运行时模块都会共享的轻量状态：哪些实体属于生成的
//! View，哪些根节点支持热重载，以及哪些布局资源正在等待重建。这些类型本身
//! 不执行逻辑，但它们是重载、初始化和更新系统能够围绕同一批 View 实体协作的
//! 基础胶水。

use super::super::layout::ViewLayoutAsset;
use bevy::prelude::*;

/// Marker component for entities that are part of a RON-driven view.
///
/// RON 驱动视图中实体的标记组件。
#[derive(Component)]
pub struct RonDrivenView;

/// Component marking an entity as a hot-reloadable view root.
/// Any entity with this component can have its view layout hot-reloaded.
///
/// 标记实体为可热重载视图根的组件。
/// 任何带有此组件的实体都可以热重载其视图布局。
#[derive(Component)]
pub struct HotReloadableViewRoot {
    /// The asset path to the view layout file
    ///
    /// 视图布局文件的资源路径
    pub layout_path: String,
    /// Handle to the currently loaded layout asset
    ///
    /// 当前已加载布局资源的句柄
    pub layout_handle: Handle<ViewLayoutAsset>,
    /// FRE events applied before the initial entity tree is spawned.
    ///
    /// 初次生成实体树前先应用的 FRE 事件。
    pub pre_spawn_events: Vec<String>,
    /// Required FRE assets kept alive while pre-spawn events wait for loading.
    ///
    /// 预生成事件等待加载时需要保持存活的 required FRE 资源。
    pub pre_spawn_fre_handles: Vec<Handle<crate::core::game_action::GameFreAsset>>,
}

/// Resource that tracks pending view reloads.
/// Maps asset IDs to sets of entities that need to be rebuilt.
///
/// 跟踪待处理视图重载的资源。
/// 将资源 ID 映射到需要重建的实体集合。
#[derive(Resource, Default)]
pub struct PendingViewReloads {
    /// Set of asset IDs that have been modified and need reload
    ///
    /// 已修改且需要重载的资源 ID 集合
    pub modified_assets: std::collections::HashSet<bevy::asset::AssetId<ViewLayoutAsset>>,
}

impl PendingViewReloads {
    /// Mark an asset as needing reload
    ///
    /// 标记资源需要重载
    pub fn mark_for_reload(&mut self, id: bevy::asset::AssetId<ViewLayoutAsset>) {
        self.modified_assets.insert(id);
    }

    /// Check if any reloads are pending
    ///
    /// 检查是否有待处理的重载
    pub fn has_pending(&self) -> bool {
        !self.modified_assets.is_empty()
    }

    /// Clear pending reload for a specific asset
    ///
    /// 清除特定资源的待处理重载
    pub fn clear(&mut self, id: &bevy::asset::AssetId<ViewLayoutAsset>) {
        self.modified_assets.remove(id);
    }

    /// Take all pending reloads
    ///
    /// 获取所有待处理的重载并清空
    pub fn take_all(&mut self) -> std::collections::HashSet<bevy::asset::AssetId<ViewLayoutAsset>> {
        std::mem::take(&mut self.modified_assets)
    }
}

/// Marker component indicating the view has been generated for this entity.
///
/// 标记组件，表示此实体的视图已生成。
#[derive(Component)]
pub struct ViewGenerated;
