//! # render_layers.rs
//!
//! # render_layers.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides centralized constants for Z-axis layer management in rendering.
//! Having all Z-index values in one place makes it easy to adjust rendering order
//! and prevents "magic number" bugs across the codebase.
//!
//! 本模块提供渲染中 Z 轴图层管理的集中常量。
//! 将所有 Z 索引值放在一处可以方便地调整渲染顺序，
//! 并防止代码库中出现"魔法数字"错误。

/// Z-axis layer constants for consistent rendering order.
///
/// Z 轴图层常量，用于保持一致的渲染顺序。
///
/// Lower values are rendered behind higher values.
/// 较低的值渲染在较高值的后面。
///
/// ## Layer Hierarchy (back to front)
///
/// ## 图层层级（从后到前）
///
/// ```text
/// Background (-100.0) ← Far background elements
/// Tilemap Base (-10.0) ← Ground tiles
/// Tilemap Layers (-2.0 base, 0.5 step) ← Tile layers sorted by index
/// Objects (0.0) ← Interactive objects, NPCs
/// Player (1.0) ← Player character
/// Effects (5.0) ← Particles, visual effects
/// UI Background (10.0) ← UI panels, dialogs
/// UI Foreground (20.0) ← UI text, buttons
/// Overlay (100.0) ← Fade effects, transitions
/// ```
pub mod z_layers {
    // === Background Layers ===
    // === 背景图层 ===

    /// Far background elements (parallax, sky).
    ///
    /// 远背景元素（视差、天空）。
    pub const BACKGROUND: f32 = -100.0;

    // === Tilemap Layers ===
    // === 瓦片地图图层 ===

    /// Base offset applied to all tilemap layers by TiledMapLayerZOffset.
    /// This value is passed to `TiledMapLayerZOffset` component.
    ///
    /// 由 TiledMapLayerZOffset 应用于所有瓦片地图图层的基础偏移。
    /// 此值传递给 `TiledMapLayerZOffset` 组件。
    pub const TILEMAP_OFFSET: f32 = 10.0;

    /// Base Z value for individual tilemap layer sorting.
    /// Layers are sorted relative to this value.
    ///
    /// 单个瓦片地图图层排序的基准 Z 值。
    /// 图层相对于此值进行排序。
    pub const TILEMAP_LAYER_BASE: f32 = -2.0;

    /// Z step between consecutive tilemap layers.
    /// Each layer is offset by this amount * index.
    ///
    /// 连续瓦片地图图层之间的 Z 步长。
    /// 每个图层偏移此量 * 索引。
    pub const TILEMAP_LAYER_STEP: f32 = 0.5;

    // === Game Object Layers ===
    // === 游戏对象图层 ===

    /// Default Z for world objects (NPCs, items, etc.).
    ///
    /// 世界对象（NPC、物品等）的默认 Z。
    pub const OBJECTS: f32 = 0.0;

    /// Player character layer (slightly in front of objects).
    ///
    /// 玩家角色图层（略微在对象前面）。
    pub const PLAYER: f32 = 1.0;

    /// Offset for objects behind the player (based on Y-sort).
    ///
    /// 玩家后面对象的偏移（基于 Y 排序）。
    pub const OBJECT_BEHIND_PLAYER: f32 = -1.0;

    /// Offset for objects in front of the player (based on Y-sort).
    ///
    /// 玩家前面对象的偏移（基于 Y 排序）。
    pub const OBJECT_IN_FRONT_OF_PLAYER: f32 = 1.0;

    // === Effect Layers ===
    // === 特效图层 ===

    /// Visual effects (particles, magic, etc.).
    ///
    /// 视觉特效（粒子、魔法等）。
    pub const EFFECTS: f32 = 5.0;

    // === UI Layers ===
    // === UI 图层 ===

    /// UI background panels and dialog boxes.
    ///
    /// UI 背景面板和对话框。
    pub const UI_BACKGROUND: f32 = 10.0;

    /// UI foreground elements (text, buttons).
    ///
    /// UI 前景元素（文本、按钮）。
    pub const UI_FOREGROUND: f32 = 20.0;

    // === Overlay Layers ===
    // === 覆盖图层 ===

    /// Fullscreen overlays (fade, transitions).
    ///
    /// 全屏覆盖（淡入淡出、转场）。
    pub const OVERLAY: f32 = 100.0;
}

/// Calculate the Z offset for a specific tilemap layer by its index.
///
/// 根据索引计算特定瓦片地图图层的 Z 偏移。
///
/// # Arguments
///
/// * `layer_index` - The layer's index (0-based)
/// * `total_layers` - Total number of layers in the map
///
/// # Returns
///
/// The calculated Z offset for this layer
///
/// # Example
///
/// ```ignore
/// // For a map with 5 layers, layer 0 would be furthest back
/// let z = calculate_tilemap_layer_z(0, 5);
/// ```
#[inline]
pub fn calculate_tilemap_layer_z(layer_index: usize, total_layers: usize) -> f32 {
    z_layers::TILEMAP_LAYER_BASE
        - (total_layers as f32 - 1.0 - layer_index as f32) * z_layers::TILEMAP_LAYER_STEP
}

/// Calculate the Z offset for a specific tilemap layer using configurable values.
///
/// 使用可配置的值计算特定瓦片地图图层的 Z 偏移。
///
/// This variant allows using values from `RenderConfig` resource.
///
/// 此变体允许使用来自 `RenderConfig` 资源的值。
#[inline]
pub fn calculate_tilemap_layer_z_with_config(
    layer_index: usize,
    total_layers: usize,
    z_base: f32,
    z_step: f32,
) -> f32 {
    z_base - (total_layers as f32 - 1.0 - layer_index as f32) * z_step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilemap_layer_z_ordering() {
        // Layer 0 should be behind layer 4 (lower Z)
        let z0 = calculate_tilemap_layer_z(0, 5);
        let z4 = calculate_tilemap_layer_z(4, 5);
        assert!(z0 < z4, "Layer 0 should be behind layer 4");
    }

    #[test]
    fn test_tilemap_layer_z_step() {
        // Adjacent layers should differ by TILEMAP_LAYER_STEP
        let z0 = calculate_tilemap_layer_z(0, 5);
        let z1 = calculate_tilemap_layer_z(1, 5);
        let diff = (z1 - z0).abs();
        assert!(
            (diff - z_layers::TILEMAP_LAYER_STEP).abs() < 0.001,
            "Adjacent layers should be separated by TILEMAP_LAYER_STEP"
        );
    }
}
