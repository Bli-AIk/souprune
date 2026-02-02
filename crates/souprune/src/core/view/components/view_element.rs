//! View Element System with history tracking for undo/redo/reset.
//!
//! 视图元素系统，带有撤销/重做/重置的历史跟踪。

use bevy::prelude::*;
use bevy_fact_rule_event::FactDatabase;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

/// View Element - represents a referenceable element in a view layout.
///
/// Each element spawned from a `.view_layout.ron` file receives this component,
/// enabling runtime queries and modifications.
///
/// 视图元素 - 表示视图布局中可被引用的元素。
///
/// 从 `.view_layout.ron` 文件生成的每个元素都会获得此组件，
/// 从而支持运行时查询和修改。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewElement {
    /// Fully qualified name with namespace.
    /// Format: "namespace::element_name"
    /// Example: "battle_ui_undertale::BtnFight"
    ///
    /// 完全限定名称（含命名空间）。
    /// 格式: "namespace::element_name"
    /// 示例: "battle_ui_undertale::BtnFight"
    pub full_name: String,

    /// Local name without namespace.
    ///
    /// 局部名称（无命名空间）。
    pub local_name: String,

    /// Namespace (inherited from ViewRoot).
    ///
    /// 命名空间（从 ViewRoot 继承）。
    pub namespace: String,

    /// Tag list for batch queries.
    ///
    /// 标签列表（用于批量查询）。
    pub tags: Vec<String>,
}

/// View Root - marks the root entity of a view layout and defines its namespace.
///
/// 视图根 - 标记视图布局的根实体并定义其命名空间。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewRoot {
    /// Layout asset path.
    ///
    /// 布局资源路径。
    pub layout_path: String,

    /// Namespace (auto-generated from layout path).
    /// Example: "battle/ui/undertale.view_layout.ron" -> "battle_ui_undertale"
    ///
    /// 命名空间（从布局路径自动生成）。
    /// 示例: "battle/ui/undertale.view_layout.ron" -> "battle_ui_undertale"
    pub namespace: String,

    /// Local fact storage for this View instance.
    /// Automatically cleared when the View is despawned.
    ///
    /// 此 View 实例的局部事实存储。
    /// 当 View 被销毁时自动清空。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub local_facts: FactDatabase,
}

impl ViewRoot {
    /// Create a new ViewRoot from a layout path.
    ///
    /// 从布局路径创建新的 ViewRoot。
    pub fn new(layout_path: String) -> Self {
        let namespace = Self::namespace_from_path(&layout_path);
        Self {
            layout_path,
            namespace,
            local_facts: FactDatabase::new(),
        }
    }

    /// Generate namespace from layout path.
    ///
    /// Removes the `.view_layout.ron` extension and replaces `/` and `.` with `_`.
    ///
    /// 从布局路径生成命名空间。
    ///
    /// 移除 `.view_layout.ron` 扩展名，并将 `/` 和 `.` 替换为 `_`。
    pub fn namespace_from_path(path: &str) -> String {
        path.trim_end_matches(".view_layout.ron")
            .replace(['/', '.'], "_")
    }
}

/// Query helper functions for ViewElement.
///
/// ViewElement 的查询辅助函数。
impl ViewElement {
    /// Create a new ViewElement.
    ///
    /// 创建新的 ViewElement。
    pub fn new(namespace: String, local_name: String, tags: Vec<String>) -> Self {
        let full_name = format!("{}::{}", namespace, local_name);
        Self {
            full_name,
            local_name,
            namespace,
            tags,
        }
    }
}

/// Find an element by its fully qualified name.
///
/// 通过完全限定名称查找元素。
pub fn find_element_by_full_name(
    query: &Query<(Entity, &ViewElement)>,
    full_name: &str,
) -> Option<Entity> {
    query
        .iter()
        .find(|(_, elem)| elem.full_name == full_name)
        .map(|(entity, _)| entity)
}

/// Find an element within a specific namespace by its local name.
///
/// 在特定命名空间内通过局部名称查找元素。
pub fn find_element_in_namespace(
    query: &Query<(Entity, &ViewElement)>,
    namespace: &str,
    local_name: &str,
) -> Option<Entity> {
    let full_name = format!("{}::{}", namespace, local_name);
    find_element_by_full_name(query, &full_name)
}

/// Find all elements with a specific tag.
///
/// 查找所有具有特定标签的元素。
pub fn find_elements_by_tag(query: &Query<(Entity, &ViewElement)>, tag: &str) -> Vec<Entity> {
    query
        .iter()
        .filter(|(_, elem)| elem.tags.contains(&tag.to_string()))
        .map(|(entity, _)| entity)
        .collect()
}

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
    ) -> Self {
        Self {
            transform: transform.map(|t| (t.translation, t.rotation, t.scale)),
            color: sprite.map(|s| s.color),
            visibility: visibility.copied(),
            texture: None,
        }
    }
}
