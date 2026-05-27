//! View root and local state components.
//!
//! View 根节点与局部状态组件。

use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy::reflect::Reflect;
use bevy_fact_rule_event::{FactDatabase, FactReader, FactValue};

use crate::core::fre_facts;
use crate::core::input::{Direction, InputCommand};

/// View-local state owned by a view root.
///
/// View 根节点拥有的局部状态。
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct LocalState {
    facts: FactDatabase,
}

impl LocalState {
    /// Create an empty local state.
    ///
    /// 创建空局部状态。
    pub fn new() -> Self {
        Self {
            facts: FactDatabase::new(),
        }
    }

    /// Read a raw fact value by key.
    ///
    /// 按 key 读取原始 fact 值。
    pub fn get_by_str(&self, key: &str) -> Option<&FactValue> {
        self.facts.get_by_str(key)
    }

    /// Read an integer fact value.
    ///
    /// 读取整数 fact 值。
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.facts.get_int(key)
    }

    /// Read a float fact value.
    ///
    /// 读取浮点 fact 值。
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.facts.get_float(key)
    }

    /// Read a boolean fact value.
    ///
    /// 读取布尔 fact 值。
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.facts.get_bool(key)
    }

    /// Read a string fact value.
    ///
    /// 读取字符串 fact 值。
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.facts.get_string(key)
    }

    /// Read a string list fact value.
    ///
    /// 读取字符串列表 fact 值。
    pub fn get_string_list(&self, key: &str) -> Option<&[String]> {
        self.facts
            .get_by_str(key)
            .and_then(FactValue::as_string_list)
    }

    /// Read an integer list fact value.
    ///
    /// 读取整数列表 fact 值。
    pub fn get_int_list(&self, key: &str) -> Option<&[i64]> {
        self.facts.get_by_str(key).and_then(FactValue::as_int_list)
    }

    /// Check whether a fact exists.
    ///
    /// 检查 fact 是否存在。
    pub fn contains(&self, key: &str) -> bool {
        self.facts.contains(key)
    }

    /// Check whether this state has no facts.
    ///
    /// 检查此状态是否没有 fact。
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Iterate through facts for read-only inspection.
    ///
    /// 以只读方式遍历 fact。
    pub fn iter(&self) -> impl Iterator<Item = (&String, &FactValue)> {
        self.facts.iter()
    }

    /// Borrow the current FactDatabase representation.
    ///
    /// 借用当前的 FactDatabase 表示。
    pub(crate) fn as_facts(&self) -> &FactDatabase {
        &self.facts
    }

    /// Mutably borrow the current FactDatabase representation for owners.
    ///
    /// 为状态拥有者可变借用当前的 FactDatabase 表示。
    pub(crate) fn as_facts_mut_for_owner(&mut self) -> &mut FactDatabase {
        &mut self.facts
    }

    /// Set a fact value from a state owner.
    ///
    /// 由状态拥有者设置 fact 值。
    pub(crate) fn set(&mut self, key: impl Into<String>, value: impl Into<FactValue>) {
        self.facts.set(key, value);
    }

    /// Remove a fact value from a state owner.
    ///
    /// 由状态拥有者移除 fact 值。
    pub(crate) fn remove(&mut self, key: &str) -> Option<FactValue> {
        self.facts.remove(key)
    }

    /// Clear all facts from a state owner.
    ///
    /// 由状态拥有者清空所有 fact。
    pub(crate) fn clear(&mut self) {
        self.facts.clear();
    }
}

impl FactReader for LocalState {
    fn get_by_str(&self, key: &str) -> Option<&FactValue> {
        self.facts.get_by_str(key)
    }

    fn contains(&self, key: &str) -> bool {
        self.facts.contains(key)
    }
}

/// View Root - marks the root entity of a view layout and defines its namespace.
///
/// The local state store is intentionally private; use `local_state()` for reads and
/// owner-only control methods for writes.
///
/// ```compile_fail
/// use souprune::core::view::ViewRoot;
///
/// let view_root = ViewRoot::new("battle/menu.view.ron".to_string());
/// // Direct field access is intentionally unavailable.
/// let _ = &view_root.local_state;
/// ```
///
/// ```compile_fail
/// use bevy_fact_rule_event::FactValue;
/// use souprune::core::view::ViewRoot;
///
/// let view_root = ViewRoot::new("battle/menu.view.ron".to_string());
/// view_root.local_state().set("selection", FactValue::Int(1));
/// ```
///
/// 视图根 - 标记视图布局的根实体并定义其命名空间。
///
/// 局部状态存储刻意保持私有；读取请使用 `local_state()`，写入只能通过
/// owner 专用控制方法。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewRoot {
    /// Layout asset path.
    ///
    /// 布局资源路径。
    pub layout_path: String,

    /// Namespace (auto-generated from layout path).
    /// Example: "battle/ui/main.view.ron" -> "battle_ui_main"
    ///
    /// 命名空间（从布局路径自动生成）。
    /// 示例: "battle/ui/main.view.ron" -> "battle_ui_main"
    pub namespace: String,

    /// Local state storage for this View instance.
    /// Automatically cleared when the View is despawned.
    ///
    /// 此 View 实例的局部状态存储。
    /// 当 View 被销毁时自动清空。
    local_state: LocalState,
}

impl ViewRoot {
    /// Local fact storing the latest View-owned navigation direction.
    ///
    /// 保存最近一次 View 自有导航方向的局部事实。
    pub const INPUT_NAVIGATION: &'static str = "view:input:navigation";

    /// Local fact set when the View receives a confirm request.
    ///
    /// View 收到确认请求时设置的局部事实。
    pub const INPUT_CONFIRM_REQUESTED: &'static str = "view:input:confirm_requested";

    /// Local fact set when the View receives a cancel request.
    ///
    /// View 收到取消请求时设置的局部事实。
    pub const INPUT_CANCEL_REQUESTED: &'static str = "view:input:cancel_requested";

    /// Local fact set when the View receives a menu request.
    ///
    /// View 收到菜单请求时设置的局部事实。
    pub const INPUT_MENU_REQUESTED: &'static str = "view:input:menu_requested";

    /// Create a new ViewRoot from a layout path.
    ///
    /// 从布局路径创建新的 ViewRoot。
    pub fn new(layout_path: String) -> Self {
        let namespace = Self::namespace_from_path(&layout_path);
        Self {
            layout_path,
            namespace,
            local_state: LocalState::new(),
        }
    }

    /// Read this view's local state.
    ///
    /// 读取此 View 的局部状态。
    pub fn local_state(&self) -> &LocalState {
        &self.local_state
    }

    /// Mutably access local state from an owning system.
    ///
    /// 从状态拥有系统可变访问局部状态。
    pub(crate) fn local_state_mut_for_owner(&mut self) -> &mut LocalState {
        &mut self.local_state
    }

    /// Set a local state value from a View-owning system.
    ///
    /// 从 View 拥有系统设置局部状态值。
    pub(crate) fn set_local_value(&mut self, key: impl Into<String>, value: impl Into<FactValue>) {
        self.local_state.set(key, value);
    }

    /// Remove a local state value from a View-owning system.
    ///
    /// 从 View 拥有系统移除局部状态值。
    pub(crate) fn remove_local_value(&mut self, key: &str) -> Option<FactValue> {
        self.local_state.remove(key)
    }

    /// Request this View to close through its controlled state channel.
    ///
    /// 通过受控状态通道请求关闭此 View。
    pub(crate) fn request_close(&mut self) {
        self.set_local_value(fre_facts::VIEW_CLOSE_REQUESTED, FactValue::Bool(true));
    }

    /// Request a sequence sub-state switch through its controlled state channel.
    ///
    /// 通过受控状态通道请求切换序列子状态。
    pub(crate) fn switch_state(&mut self, state_name: impl Into<String>) {
        self.set_local_value(
            fre_facts::VIEW_SWITCH_STATE,
            FactValue::String(state_name.into()),
        );
    }

    /// Override a local fact from debug tooling.
    ///
    /// 从调试工具覆盖局部 fact。
    pub fn override_local_value_for_debug(
        &mut self,
        key: impl Into<String>,
        value: impl Into<FactValue>,
    ) {
        self.local_state.set(key, value);
    }

    /// Clear local state from debug tooling as an explicit override.
    ///
    /// 作为显式调试覆盖清空局部状态。
    pub fn clear_local_state_for_debug_override(&mut self) {
        self.local_state.clear();
    }

    /// Generate namespace from layout path.
    ///
    /// Removes the `.view.ron` extension and replaces `/` and `.` with `_`.
    ///
    /// 从布局路径生成命名空间。
    ///
    /// 移除 `.view.ron` 扩展名，并将 `/` 和 `.` 替换为 `_`。
    pub fn namespace_from_path(path: &str) -> String {
        path.trim_end_matches(".view.ron").replace(['/', '.'], "_")
    }

    /// Apply a semantic input command through View-owned control methods.
    ///
    /// 通过 View 自有的受控方法应用语义输入命令。
    pub fn apply_input_command(&mut self, command: &InputCommand) {
        match command {
            InputCommand::Navigate(direction) => self.request_navigation(*direction),
            InputCommand::Confirm => self.request_confirm(),
            InputCommand::Cancel => self.request_cancel(),
            InputCommand::Menu => self.request_menu(),
        }
    }

    /// Request View-owned navigation.
    ///
    /// 请求 View 自有导航。
    pub fn request_navigation(&mut self, direction: Direction) {
        let direction_name = match direction {
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Left => "left",
            Direction::Right => "right",
        };
        self.local_state.set(Self::INPUT_NAVIGATION, direction_name);

        if let Some(selection) = self.local_state.get_int("selection") {
            let next = match direction {
                Direction::Up | Direction::Left => selection - 1,
                Direction::Down | Direction::Right => selection + 1,
            };
            self.local_state.set("selection", FactValue::Int(next));
        }
    }

    /// Request View-owned confirmation.
    ///
    /// 请求 View 自有确认。
    pub fn request_confirm(&mut self) {
        self.local_state
            .set(Self::INPUT_CONFIRM_REQUESTED, FactValue::Bool(true));
        if self.local_state.contains("confirm_pressed") {
            self.local_state
                .set("confirm_pressed", FactValue::Bool(true));
        }
    }

    /// Request View-owned cancellation.
    ///
    /// 请求 View 自有取消。
    pub fn request_cancel(&mut self) {
        self.local_state
            .set(Self::INPUT_CANCEL_REQUESTED, FactValue::Bool(true));
    }

    /// Request View-owned menu activation.
    ///
    /// 请求 View 自有菜单激活。
    pub fn request_menu(&mut self) {
        self.local_state
            .set(Self::INPUT_MENU_REQUESTED, FactValue::Bool(true));
    }
}
