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
