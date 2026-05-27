//! View element history components.
//!
//! View 元素历史组件。

use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

/// View Element History - tracks modification history for undo/redo/reset.
///
/// 视图元素历史 - 跟踪修改历史以支持撤销/重做/重置。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewElementHistory {
    /// Original state when element was first spawned.
    ///
    /// 元素首次生成时的原始状态。
    pub original: ElementState,

    /// History stack of past states (for undo).
    ///
    /// 过去状态的历史栈（用于撤销）。
    pub history: Vec<ElementState>,

    /// Redo stack of undone states (for redo).
    ///
    /// 已撤销状态的重做栈（用于重做）。
    pub redo_stack: Vec<ElementState>,

    /// Current index in history (-1 means at original state).
    ///
    /// 历史中的当前索引（-1 表示处于原始状态）。
    pub current_index: isize,
}

/// Element State - snapshot of an element's mutable properties.
///
/// 元素状态 - 元素可变属性的快照。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ElementState {
    /// Transform (position, rotation, scale).
    ///
    /// 变换（位置、旋转、缩放）。
    pub transform: Option<(Vec3, Quat, Vec3)>,

    /// Sprite color.
    ///
    /// 精灵颜色。
    pub color: Option<Color>,

    /// Visibility.
    ///
    /// 可见性。
    pub visibility: Option<Visibility>,

    /// Texture path (if Sprite has image).
    ///
    /// 贴图路径（如果 Sprite 有图片）。
    pub texture: Option<String>,

    /// ViewBox alpha (if ViewBox present).
    ///
    /// ViewBox 透明度（如果有 ViewBox）。
    pub view_box_alpha: Option<f32>,
}

impl ViewElementHistory {
    /// Create a new history tracker with the given original state.
    ///
    /// 使用给定的原始状态创建新的历史跟踪器。
    pub fn new(original: ElementState) -> Self {
        Self {
            original,
            history: Vec::new(),
            redo_stack: Vec::new(),
            current_index: -1,
        }
    }

    /// Push a new state to history (called AFTER a modification is made).
    ///
    /// This should be called with the NEW state after applying a modification.
    ///
    /// 将新状态推送到历史（在进行修改后调用）。
    ///
    /// 应该在应用修改后使用新状态调用。
    pub fn push(&mut self, new_state: ElementState) {
        self.redo_stack.clear();

        if self.current_index >= 0 {
            self.history.truncate((self.current_index + 1) as usize);
        } else {
            self.history.clear();
        }

        self.history.push(new_state);
        self.current_index = self.history.len() as isize - 1;
    }

    /// Undo last modification, returns the previous state.
    ///
    /// 撤销最后一次修改，返回之前的状态。
    pub fn undo(&mut self) -> Option<ElementState> {
        if self.current_index < 0 {
            return None;
        }

        let current = self.history[self.current_index as usize].clone();
        self.redo_stack.push(current);
        self.current_index -= 1;

        if self.current_index >= 0 {
            Some(self.history[self.current_index as usize].clone())
        } else {
            Some(self.original.clone())
        }
    }

    /// Redo last undone modification, returns the next state.
    ///
    /// 重做最后撤销的修改，返回下一个状态。
    pub fn redo(&mut self) -> Option<ElementState> {
        if let Some(state) = self.redo_stack.pop() {
            self.current_index += 1;
            Some(state)
        } else {
            None
        }
    }

    /// Reset to original state.
    ///
    /// 重置为原始状态。
    pub fn reset(&mut self) -> ElementState {
        self.current_index = -1;
        self.redo_stack.clear();
        self.original.clone()
    }

    /// Get debug info about history stack sizes.
    ///
    /// 获取历史栈大小的调试信息。
    pub fn debug_info(&self) -> (usize, usize, isize) {
        (
            self.history.len(),
            self.redo_stack.len(),
            self.current_index,
        )
    }
}

impl ElementState {
    /// Capture current state from entity components.
    ///
    /// 从实体组件捕获当前状态。
    pub fn capture(
        transform: Option<&Transform>,
        sprite: Option<&Sprite>,
        visibility: Option<&Visibility>,
        view_box: Option<&crate::core::view::components::box_components::ViewBox>,
    ) -> Self {
        Self {
            transform: transform.map(|t| (t.translation, t.rotation, t.scale)),
            color: sprite.map(|s| s.color),
            visibility: visibility.copied(),
            texture: None,
            view_box_alpha: view_box.map(|vb| vb.alpha()),
        }
    }
}
