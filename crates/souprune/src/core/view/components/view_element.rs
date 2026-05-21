//! View Element System with history tracking for undo/redo/reset.
//!
//! 视图元素系统，带有撤销/重做/重置的历史跟踪。

use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

/// Marker component for the currently active View.
///
/// When a View enters an interactive state (e.g., menu opened),
/// it receives this marker. FRE actions like `SetLocalFact` and
/// `CloseView` only affect ViewRoots with this marker.
///
/// This prevents "crosstalk" when multiple Views exist simultaneously.
///
/// 当前活跃 View 的标记组件。
///
/// 当 View 进入交互状态（例如菜单打开）时，
/// 它会获得此标记。`SetLocalFact` 和 `CloseView` 等 FRE 动作
/// 仅影响带有此标记的 ViewRoot。
///
/// 这可以防止多个 View 同时存在时的"串台"问题。
#[derive(Component, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ActiveView;

/// Marker component for View roots that participate in the View focus stack.
///
/// Views receive this marker when they can own View-targeted input focus.
/// `ActiveView` is derived from the top of `ViewFocusStack`; systems should
/// insert this scope marker instead of inserting `ActiveView` directly.
///
/// 参与 View 焦点栈的 View 根节点标记组件。
///
/// 当 View 可以拥有面向 View 的输入焦点时，它会获得此标记。
/// `ActiveView` 由 `ViewFocusStack` 栈顶派生；系统应插入此作用域标记，
/// 而不是直接插入 `ActiveView`。
#[derive(Component, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewFocusScope;

/// Authoritative stack of View roots that can receive View-targeted input.
///
/// The newest pushed entity is the current focus owner. Pushing an existing
/// entity moves it to the top instead of duplicating it.
///
/// 可接收面向 View 输入的 View 根节点权威栈。
///
/// 最新推入的实体是当前焦点拥有者。推入已存在实体会把它移动到栈顶，
/// 而不是创建重复项。
#[derive(Resource, Debug, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewFocusStack {
    stack: Vec<Entity>,
}

impl ViewFocusStack {
    /// Push an entity onto the focus stack, moving existing entries to the top.
    ///
    /// 将实体推入焦点栈；如果实体已存在，则移动到栈顶。
    pub fn push(&mut self, entity: Entity) {
        self.remove(entity);
        self.stack.push(entity);
    }

    /// Remove an entity from the focus stack.
    ///
    /// 从焦点栈中移除实体。
    pub fn remove(&mut self, entity: Entity) -> Option<Entity> {
        let position = self
            .stack
            .iter()
            .position(|candidate| *candidate == entity)?;
        Some(self.stack.remove(position))
    }

    /// Return the current focus owner.
    ///
    /// 返回当前焦点拥有者。
    pub fn top(&self) -> Option<Entity> {
        self.stack.last().copied()
    }

    /// Clear all focus ownership.
    ///
    /// 清空所有焦点归属。
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(Entity) -> bool) {
        self.stack.retain(|entity| keep(*entity));
    }
}

/// Tags from a `.view_layout.ron` node definition.
///
/// Runtime systems can query for entities with specific tags and add
/// mode-specific components accordingly.
///
/// 来自 `.view_layout.ron` 节点定义的标签。
///
/// 运行时系统可以查询具有特定标签的实体，并据此添加模式专属组件。
#[derive(Component, Debug, Clone)]
pub struct ViewNodeTags(pub Vec<String>);

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
    /// Example: "battle_ui_main::BtnFight"
    ///
    /// 完全限定名称（含命名空间）。
    /// 格式: "namespace::element_name"
    /// 示例: "battle_ui_main::BtnFight"
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

mod history;
mod root;

pub use history::{ElementState, ViewElementHistory};
pub use root::{LocalState, ViewRoot};

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

/// Component that stores a `visible_when` expression for runtime visibility evaluation.
///
/// The expression is evaluated every frame against the current fact state.
///
/// 存储 `visible_when` 表达式以进行运行时可见性评估的组件。
///
/// 表达式每帧根据当前 fact 状态进行评估。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct VisibleWhen {
    /// The expression to evaluate for visibility.
    /// Examples: "true", "$depth == 0", "fact('selection') == 1"
    ///
    /// 用于评估可见性的表达式。
    /// 示例: "true", "$depth == 0", "fact('selection') == 1"
    pub expression: String,
}

/// Component to track pending View rules that need to be registered
/// when their FRE assets finish loading.
///
/// This enables delayed rule registration for View-scoped rules,
/// handling the timing issue where FRE assets may not be loaded
/// when the View is first spawned.
///
/// 跟踪待注册的 View 规则的组件，当 FRE 资源加载完成后注册。
///
/// 这实现了 View 作用域规则的延迟注册，
/// 处理 View 首次生成时 FRE 资源可能还未加载的时序问题。
#[derive(Component, Debug, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct PendingViewRules {
    /// FRE file handles waiting for asset loading.
    /// Storing handles (not paths) keeps the asset loading request alive.
    ///
    /// 等待资源加载的 FRE 文件句柄。
    /// 存储句柄（而非路径）可保持资源加载请求不被取消。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub pending_handles: Vec<(
        String,
        bevy::prelude::Handle<crate::core::game_action::GameFreAsset>,
    )>,
}

/// Per-entity component for view data bindings.
/// Stores bindings and FRE asset handles that need to be loaded before view spawn.
///
/// 每实体的视图数据绑定组件。
/// 存储在视图生成前需要加载的绑定和 FRE 资源句柄。
#[derive(Component, Debug, Clone, Default)]
pub struct PendingViewData {
    /// Data bindings for this view (interface name → binding).
    pub bindings:
        std::collections::HashMap<String, crate::core::sequencer::chapter_schema::DataBinding>,
    /// Handles to FRE assets being loaded for bindings (keeps loading alive).
    pub fre_handles: Vec<bevy::prelude::Handle<crate::core::game_action::GameFreAsset>>,
}

#[cfg(test)]
mod tests;
