//! Overworld UI components used by the overworld app state.
//!
//! 用于 overworld 应用状态的 UI 组件。
//!
//! Components and helpers keep track of the active UI layer plus the selected index inside it.
//!
//! 这些组件用于跟踪当前激活的 UI 层以及该层内被选择的索引。
//!
//! Fields remain private and are accessed through read-only getters and guarded setters.
//!
//! 字段保持私有，只能通过只读 getter 和受控 setter 访问与修改。

use crate::core::input::Action;
use bevy::color::Srgba;
use bevy::prelude::{
    Bundle, Color, Component, Entity, Name, Query, Resource, Sprite, Transform, Vec2, Vec3, Quat, Visibility,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;
use bevy_rich_text3d::{TextAlign, TextAnchor};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct UILayer(Cow<'static, str>);

impl UILayer {
    pub const BACKPACK_MENU: UILayer = UILayer::new_static("BackpackMenu");
    pub const BACKPACK_ITEM: UILayer = UILayer::new_static("BackpackItem");
    pub const BACKPACK_ITEM_CHOOSES: UILayer = UILayer::new_static("BackpackItemOptions");
    pub const BACKPACK_STATUS: UILayer = UILayer::new_static("BackpackStatus");

    /// Defined options for the backpack menu, determining order and count.
    ///
    /// 背包菜单的定义选项，决定顺序和数量。
    pub const BACKPACK_MENU_OPTIONS: &'static [UILayer] =
        &[Self::BACKPACK_ITEM, Self::BACKPACK_STATUS];

    /// Const constructor for static constants
    ///
    /// Const 构造函数，用于静态常量初始化
    const fn new_static(name: &'static str) -> UILayer {
        UILayer(Cow::Borrowed(name))
    }

    /// Dynamically construct a layer (flexible for mods or expansions)
    ///
    /// 动态构造层（灵活扩展）
    pub fn new(name: impl Into<Cow<'static, str>>) -> UILayer {
        UILayer(name.into())
    }

    /// Get the layer name
    ///
    /// 获取层名称
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UILayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Options available when selecting an item in the backpack.
///
/// 背包中选中物品时可用的选项。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BackpackItemOption {
    Use = 0,
    Info = 1,
    Drop = 2,
}

impl BackpackItemOption {
    /// All available item options in order.
    ///
    /// 按顺序排列的所有可用物品选项。
    pub const ALL: &'static [Self] = &[Self::Use, Self::Info, Self::Drop];

    /// Get the total count of item options.
    ///
    /// 获取物品选项的总数。
    pub const fn count() -> usize {
        Self::ALL.len()
    }
}

/// Component that records the UI layer and the current selection index within that layer.
///
/// Access pattern:
/// - Fields are private to enforce read-only access from outside code in the crate.
/// - Use the provided getters to read `layer`, `index` and `max_index`.
/// - Use `set_layer` and `set_index` to change state in a controlled way (clamps and resets index as needed).
///
/// 记录 UI 层以及该层内当前选中项索引的组件。
///
/// 访问约定：
/// - 字段为私有以在 crate 范围内强制读取访问。
/// - 使用提供的 getter 来读取 `layer`、`index` 和 `max_index`。
/// - 使用 `set_layer` 和 `set_index` 以受控方式修改状态（会进行夹住或重置索引）。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIAnimationState {
    pub(crate) state_name: String,
}

#[derive(Component, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct RonUI {
    layer: UILayer,
    index: usize,
    max_index: usize,
}

impl RonUI {
    /// Create a new `RonUI` component for `layer` with the given `max_index`.
    ///
    /// 为指定的 `layer` 创建一个新的 `RonUI` 组件，并设置 `max_index`。
    pub(crate) fn new(layer: UILayer, max_index: usize) -> Self {
        Self {
            layer,
            index: 0,
            max_index,
        }
    }

    /// Get the current UI layer.
    ///
    /// 获取当前的 UI 层级。
    pub(crate) fn layer(&self) -> &UILayer {
        &self.layer
    }

    /// Get the current selected index inside the active layer.
    ///
    /// 获取当前在激活层内所选的索引。
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    /// Get the maximum valid index for the active layer. Indexes are clamped to this value.
    ///
    /// 获取当前激活层的最大有效索引。索引会被限制在该值之内。
    #[allow(dead_code)]
    pub(crate) fn max_index(&self) -> usize {
        self.max_index
    }

    /// Change the active layer and update `max_index` accordingly.
    /// If the layer changes, the selection `index` is reset to 0. If the current `index`
    /// is greater than the new `max_index`, it will be clamped down.
    ///
    /// 更改激活层并相应地更新 `max_index`。
    /// 若层发生变化，会将选中索引 `index` 重置为 0；若当前 `index` 大于新的 `max_index`，会被夹住。
    pub(crate) fn set_layer(&mut self, layer: UILayer, max_index: usize) {
        if self.layer != layer {
            self.layer = layer;
            self.index = 0;
        }
        self.max_index = max_index;
        if self.index > self.max_index {
            self.index = self.max_index;
        }
    }

    /// Set the selection index within the current layer. The provided index will be
    /// clamped to the range [0, max_index].
    ///
    /// 设置当前层内的选择索引。提供的索引会被夹在 [0, max_index] 范围内。
    pub(crate) fn set_index(&mut self, idx: usize) {
        self.index = idx.min(self.max_index);
    }
}

/// Font configuration for UI text
///
/// UI 文本的字体配置
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) enum UIFont {
    DeterminationMono,
    DeterminationSans,
    Hud,
    BattleHud,
    // Add more fonts as needed.
    //
    // 按需继续添加更多字体。
}

impl UIFont {
    /// Get font name and default size
    ///
    /// 获取字体名称和默认大小
    pub(crate) fn font_name(&self) -> &'static str {
        match self {
            UIFont::DeterminationMono => "Determination Mono SimSun",
            UIFont::DeterminationSans => "Determination Sans SimSun",
            UIFont::Hud => "Crypt of Tomorrow Fusion",
            UIFont::BattleHud => "Mars Needs Cunnilingus",
        }
    }

    /// Get default rendering size (for texture atlas)
    ///
    /// 获取默认渲染大小（用于纹理图集）
    pub(crate) fn default_size(&self) -> f32 {
        // Rendering size affects high-resolution clarity, so we default to 128 to avoid blurry glyphs.
        //
        // 渲染大小会影响高分辨率下的清晰度，因此默认使用 128 以避免模糊的字形。
        128.
    }
}

/// Configuration for a single text element
///
/// 单个文本元素的配置
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UITextConfig {
    pub(crate) name: Name,
    pub(crate) content: String,
    pub(crate) template: Option<String>,
    pub(crate) font: UIFont,
    pub(crate) world_scale: Vec2,
    pub(crate) color: Srgba,
    pub(crate) transform: Transform,
    pub(crate) align: TextAlign,
    pub(crate) anchor: TextAnchor,
    pub(crate) line_height: f32,
}

impl Default for UITextConfig {
    fn default() -> Self {
        Self {
            name: Name::new("Text"),
            content: "Text".to_string(),
            template: None,
            font: UIFont::DeterminationMono,
            world_scale: Vec2::splat(13.),
            color: Srgba::WHITE,
            transform: Transform::default(),
            align: TextAlign::Left,
            anchor: TextAnchor::BOTTOM_RIGHT,
            line_height: 1.0,
        }
    }
}

/// Stores the original template string for dynamic text updates.
///
/// 存储原始模板字符串以用于动态文本更新。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UITextTemplate(pub(crate) String);

/// Marks UI entities that should stick to the camera with a constant offset.
///
/// 标记需要根据摄像机位置保持固定偏移的 UI 实体
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct CameraAnchored {
    pub(crate) offset: Vec3,
}

impl CameraAnchored {
    pub(crate) fn new(offset: Vec3) -> Self {
        Self { offset }
    }
}

/// Marks UI entities that should stick to the camera with a dynamic offset evaluated from expressions.
///
/// 标记需要根据从表达式评估的动态偏移量粘附在相机上的 UI 实体。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct CameraAnchoredDynamic {
    pub(crate) x_expression: Option<String>,
    pub(crate) y_expression: Option<String>,
    pub(crate) z_expression: Option<String>,
}

/// Convenience bundle to apply [`CameraAnchored`] with the correct transform in one go.
///
/// 方便的 Bundle，便于一次性添加 [`CameraAnchored`] 与正确的 Transform。
#[derive(Bundle, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct CameraAnchoredBundle {
    anchor: CameraAnchored,
    transform: Transform,
}

impl CameraAnchoredBundle {
    pub(crate) fn from_camera_transform(camera_transform: &Transform, offset: Vec3) -> Self {
        Self {
            anchor: CameraAnchored::new(offset),
            transform: Transform::from_translation(camera_transform.translation + offset),
        }
    }
}

#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIBox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) border_width: f32,
    pub(crate) texts: Vec<UITextConfig>,
    /// Optional custom fill shader path for data-driven shader loading.
    ///
    /// 可选的自定义填充着色器路径，用于数据驱动的着色器加载。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub(crate) fill_shader: Option<String>,
    /// Optional path to load a complex SmudShape structure from file.
    /// If None, generates a single SmudShape (default behavior).
    ///
    /// 可选的路径，用于从文件加载复杂的 SmudShape 结构。
    /// 如果为 None，则生成单个 SmudShape（默认行为）。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub(crate) structure_file: Option<String>,
    /// Fill color for the shape.
    ///
    /// 形状的填充颜色。
    pub(crate) fill_color: Color,
}

impl UIBox {
    /// Create a new `UIBox` component with the given dimensions and border width.
    ///
    /// 创建一个新的 `UIBox` 组件，指定尺寸和边框宽度。
    #[allow(dead_code)]
    pub(crate) fn new(width: f32, height: f32, border_width: f32) -> Self {
        Self {
            width,
            height,
            border_width,
            texts: Vec::new(),
            fill_shader: None,
            structure_file: None,
            fill_color: Color::BLACK,
        }
    }

    /// Create a new `UIBox` component with text configurations.
    ///
    /// 创建一个带有文本配置的新 `UIBox` 组件。
    #[allow(dead_code)]
    pub(crate) fn new_with_texts(
        width: f32,
        height: f32,
        border_width: f32,
        texts: Vec<UITextConfig>,
    ) -> Self {
        Self {
            width,
            height,
            border_width,
            texts,
            fill_shader: None,
            structure_file: None,
            fill_color: Color::BLACK,
        }
    }

    /// Create a new `UIBox` component with full configuration.
    ///
    /// 创建一个带有完整配置的新 `UIBox` 组件。
    pub(crate) fn new_full(
        width: f32,
        height: f32,
        border_width: f32,
        texts: Vec<UITextConfig>,
        fill_shader: Option<String>,
        structure_file: Option<String>,
        fill_color: Color,
    ) -> Self {
        Self {
            width,
            height,
            border_width,
            texts,
            fill_shader,
            structure_file,
            fill_color,
        }
    }

    /// Get the box width.
    ///
    /// 获取框的宽度。
    pub(crate) fn width(&self) -> f32 {
        self.width
    }

    /// Get the box height.
    ///
    /// 获取框的高度。
    pub(crate) fn height(&self) -> f32 {
        self.height
    }

    /// Get the border width.
    ///
    /// 获取边框宽度。
    pub(crate) fn border_width(&self) -> f32 {
        self.border_width
    }

    /// Get the custom fill shader path.
    ///
    /// 获取自定义填充着色器路径。
    #[allow(dead_code)]
    pub(crate) fn fill_shader(&self) -> Option<&str> {
        self.fill_shader.as_deref()
    }

    /// Get the structure file path.
    ///
    /// 获取结构文件路径。
    #[allow(dead_code)]
    pub(crate) fn structure_file(&self) -> Option<&str> {
        self.structure_file.as_deref()
    }

    /// Get the fill color.
    ///
    /// 获取填充颜色。
    #[allow(dead_code)]
    pub(crate) fn fill_color(&self) -> Color {
        self.fill_color
    }

    /// Set the box dimensions.
    ///
    /// 设置框的尺寸。
    #[allow(dead_code)]
    pub(crate) fn set_dimensions(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Set the border width.
    ///
    /// 设置边框宽度。
    #[allow(dead_code)]
    pub(crate) fn set_border_width(&mut self, border_width: f32) {
        self.border_width = border_width;
    }
}

/// Controls which [`UILayer`]s should render a given [`UIBox`].
///
/// 控制指定 [`UIBox`] 在哪些 [`UILayer`] 中可见。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIBoxVisibility {
    rule: UILayerVisibilityRule,
}

impl UIBoxVisibility {
    pub(crate) fn new(rule: UILayerVisibilityRule) -> Self {
        Self { rule }
    }

    pub(crate) fn rule(&self) -> &UILayerVisibilityRule {
        &self.rule
    }

    pub(crate) fn is_visible_for(&self, layer: &UILayer) -> bool {
        self.rule.is_visible_for(layer)
    }
}

/// Marker component attached to the cursor sprite entity spawned under a UI box.
///
/// 标记生成在 UI 框下方的光标精灵实体。
#[derive(Component)]
pub(crate) struct BoxCursorSprite;

/// Records which `UIBox` owns a cursor sprite entity.
///
/// 记录哪个 `UIBox` 拥有光标精灵实体。
#[derive(Component, Copy, Clone)]
pub(crate) struct BoxCursorOwner(pub Entity);

/// Marker indicating the cursor sprite has been spawned for this box.
///
/// 表示该 UI 框已经生成光标精灵的标记。
#[derive(Component, Copy, Clone)]
pub(crate) struct BoxCursorReady;

/// Marker placed on the filler entity that contains UI text and cursor sprites.
///
/// 标记承载 UI 文本与光标精灵的填充实体。
#[derive(Component)]
pub(crate) struct UIBoxFiller;

/// Marker component for a UI container node that can hold texts and children
/// without requiring a visual UIBox (background box).
///
/// 用于标记 UI 容器节点的组件，该节点可以承载文本和子节点，
/// 而无需视觉上的 UIBox（背景框）。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIContainer;

/// Controls which [`UILayer`]s should render a given UI container (without UIBox).
///
/// 控制指定 UI 容器（无 UIBox）在哪些 [`UILayer`] 中可见。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIContainerVisibility {
    rule: UILayerVisibilityRule,
}

impl UIContainerVisibility {
    pub(crate) fn new(rule: UILayerVisibilityRule) -> Self {
        Self { rule }
    }

    pub(crate) fn rule(&self) -> &UILayerVisibilityRule {
        &self.rule
    }

    pub(crate) fn is_visible_for(&self, layer: &UILayer) -> bool {
        self.rule.is_visible_for(layer)
    }
}

/// Controls the visibility behavior of a [`BoxCursor`] relative to the active [`UILayer`].
///
/// 控制 [`BoxCursor`] 相对于当前激活 [`UILayer`] 的可见性表现。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[derive(Default)]
pub(crate) enum UILayerVisibilityRule {
    #[default]
    Always,
    AlwaysHidden,
    OnlyIn(Vec<UILayer>),
    Except(Vec<UILayer>),
}

impl UILayerVisibilityRule {
    pub(crate) fn is_visible_for(&self, layer: &UILayer) -> bool {
        match self {
            UILayerVisibilityRule::Always => true,
            UILayerVisibilityRule::AlwaysHidden => false,
            UILayerVisibilityRule::OnlyIn(layers) => layers.iter().any(|l| l == layer),
            UILayerVisibilityRule::Except(layers) => layers.iter().all(|l| l != layer),
        }
    }
}

pub(crate) use UILayerVisibilityRule as BoxCursorVisibility;

/// Helper that turns an index into a translation offset for the cursor sprite.
///
/// 将索引转换为光标精灵位移的辅助类型。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) enum BoxCursorPosition {
    Static(Vec3),
    Linear { origin: Vec3, step: Vec3 },
    Custom(Vec<Vec3>),
}

impl Default for BoxCursorPosition {
    fn default() -> Self {
        BoxCursorPosition::Static(Vec3::ZERO)
    }
}

impl BoxCursorPosition {
    #[allow(dead_code)]
    pub(crate) fn fixed(position: Vec3) -> Self {
        Self::Static(position)
    }

    #[allow(dead_code)]
    pub(crate) fn linear(origin: Vec3, step: Vec3) -> Self {
        Self::Linear { origin, step }
    }

    #[allow(dead_code)]
    pub(crate) fn custom(positions: Vec<Vec3>) -> Self {
        Self::Custom(positions)
    }

    pub(crate) fn position_for_index(&self, index: usize) -> Vec3 {
        match self {
            BoxCursorPosition::Static(position) => *position,
            BoxCursorPosition::Linear { origin, step } => *origin + *step * index as f32,
            BoxCursorPosition::Custom(positions) => {
                if positions.is_empty() {
                    Vec3::ZERO
                } else {
                    positions[index.min(positions.len() - 1)]
                }
            }
        }
    }
}

/// Defines cursor placement rules, including a default strategy and layer-specific overrides.
///
/// 定义光标放置规则，包括默认策略和特定层的覆盖规则。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct BoxCursorPlacement {
    pub(crate) default: BoxCursorPosition,
    pub(crate) overrides: HashMap<UILayer, BoxCursorPosition>,
}

impl BoxCursorPlacement {
    pub(crate) fn new(default: BoxCursorPosition) -> Self {
        Self {
            default,
            overrides: HashMap::new(),
        }
    }

    pub(crate) fn with_override(mut self, layer: UILayer, position: BoxCursorPosition) -> Self {
        self.overrides.insert(layer, position);
        self
    }

    pub(crate) fn get(&self, layer: &UILayer) -> &BoxCursorPosition {
        self.overrides.get(layer).unwrap_or(&self.default)
    }
}

impl From<BoxCursorPosition> for BoxCursorPlacement {
    fn from(position: BoxCursorPosition) -> Self {
        Self::new(position)
    }
}

/// Configurable cursor that can be attached to any [`UIBox`].
///
/// 可附着在任意 [`UIBox`] 上的可配置光标。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct BoxCursor {
    pub(crate) sprite: Sprite,
    pub(crate) visibility: BoxCursorVisibility,
    pub(crate) placement: BoxCursorPlacement,
    pub(crate) transform: Transform,
    hidden: bool,
    last_index: Option<usize>,
    last_layer: Option<UILayer>,
}

impl BoxCursor {
    pub(crate) fn new(
        sprite: Sprite,
        visibility: BoxCursorVisibility,
        placement: impl Into<BoxCursorPlacement>,
        transform: Transform,
    ) -> Self {
        Self {
            sprite,
            visibility,
            placement: placement.into(),
            transform,
            hidden: false,
            last_index: None,
            last_layer: None,
        }
    }

    pub(crate) fn sprite(&self) -> Sprite {
        self.sprite.clone()
    }

    pub(crate) fn visibility(&self) -> &BoxCursorVisibility {
        &self.visibility
    }

    pub(crate) fn desired_translation(&self, layer: &UILayer, index: usize) -> Vec3 {
        let position_rule = self.placement.get(layer);
        self.transform.translation + position_rule.position_for_index(index)
    }

    pub(crate) fn transform(&self) -> Transform {
        self.transform
    }

    pub(crate) fn translation_for_index(&mut self, layer: &UILayer, index: usize) -> Option<Vec3> {
        if self.last_index == Some(index) && self.last_layer.as_ref() == Some(layer) {
            return None;
        }

        self.last_index = Some(index);
        self.last_layer = Some(layer.clone());
        Some(self.desired_translation(layer, index))
    }

    #[allow(dead_code)]
    pub(crate) fn hide(&mut self) {
        self.hidden = true;
    }

    #[allow(dead_code)]
    pub(crate) fn show(&mut self) {
        self.hidden = false;
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.hidden
    }
}

/// Describes how directional inputs should modify the index of a [`UILayer`].
///
/// 描述方向输入应如何修改 [`UILayer`] 的索引。
#[derive(Debug, Clone)]
pub(crate) struct UILayerNavigationRule {
    adjustments: HashMap<Action, isize>,
    looping: bool,
    min_index: Option<IndexBound>,
    max_index: Option<IndexBound>,
    sound_on_navigate: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum IndexBound {
    Static(usize),
    Dynamic(String),
}

impl UILayerNavigationRule {
    pub(crate) fn new(pairs: impl IntoIterator<Item = (Action, isize)>) -> Self {
        Self {
            adjustments: pairs.into_iter().collect::<HashMap<_, _>>(),
            looping: false,
            min_index: None,
            max_index: None,
            sound_on_navigate: None,
        }
    }

    pub(crate) fn new_with_bounds(
        pairs: impl IntoIterator<Item = (Action, isize)>,
        looping: bool,
        min_index: Option<IndexBound>,
        max_index: Option<IndexBound>,
        sound_on_navigate: Option<String>,
    ) -> Self {
        Self {
            adjustments: pairs.into_iter().collect::<HashMap<_, _>>(),
            looping,
            min_index,
            max_index,
            sound_on_navigate,
        }
    }

    pub(crate) fn delta_for(&self, action: Action) -> Option<isize> {
        self.adjustments.get(&action).copied()
    }

    pub(crate) fn looping(&self) -> bool {
        self.looping
    }

    pub(crate) fn min_index(&self) -> &Option<IndexBound> {
        &self.min_index
    }

    pub(crate) fn max_index(&self) -> &Option<IndexBound> {
        &self.max_index
    }

    pub(crate) fn sound_on_navigate(&self) -> Option<&str> {
        self.sound_on_navigate.as_deref()
    }
}

/// Registry that stores the navigation rules for every [`UILayer`].
///
/// 存储每个 [`UILayer`] 导航规则的注册表。
#[derive(Resource, Debug, Default)]
pub(crate) struct UILayerNavigationConfig {
    rules: HashMap<UILayer, UILayerNavigationRule>,
}

impl UILayerNavigationConfig {
    pub(crate) fn get(&self, layer: &UILayer) -> Option<&UILayerNavigationRule> {
        self.rules.get(layer)
    }

    pub(crate) fn set_rule(&mut self, layer: UILayer, rule: UILayerNavigationRule) {
        self.rules.insert(layer, rule);
    }
}

impl Default for UILayerNavigationRule {
    fn default() -> Self {
        Self::new([])
    }
}

/// Stores state transition logic for UI layers, loaded from RON configuration.
///
/// 存储 UI 层的状态转换逻辑，从 RON 配置中加载。
#[derive(Resource, Debug, Default)]
pub(crate) struct UILayerTransitionConfig {
    transitions: HashMap<UILayer, LayerTransitions>,
}

#[derive(Debug, Clone)]
pub(crate) struct LayerTransitions {
    pub(crate) on_confirm: Vec<TransitionRule>,
    pub(crate) on_cancel: Option<TransitionAction>,
    pub(crate) sound_on_confirm: Option<String>,
    pub(crate) sound_on_cancel: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct TransitionRule {
    pub(crate) condition: Option<String>,
    pub(crate) action: TransitionAction,
}

#[derive(Debug, Clone)]
pub(crate) enum TransitionAction {
    GotoLayer(UILayer),
    PopState,
    PushState(String),
}

impl UILayerTransitionConfig {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            transitions: HashMap::new(),
        }
    }

    pub(crate) fn set_transitions(&mut self, layer: UILayer, transitions: LayerTransitions) {
        self.transitions.insert(layer, transitions);
    }

    pub(crate) fn get(&self, layer: &UILayer) -> Option<&LayerTransitions> {
        self.transitions.get(layer)
    }
}

/// Marker component for HP bar sprites that need custom Material2d setup.
/// 标记组件，用于需要自定义 Material2d 设置的 HP 条精灵。
#[derive(Component)]
pub struct HPBarSprite {
    pub shader_params: Color,
}

/// Marker component for UI elements that need dynamic updates based on player data.
/// Stores the original definition for re-evaluation.
/// 标记需要根据玩家数据动态更新的UI元素的组件。
/// 存储原始定义以便重新求值。
#[derive(Component, Clone)]
pub struct DynamicUIElement {
    pub sprite_def: Option<crate::core::ui::layout::SpriteDef>,
    pub text_def: Option<crate::core::ui::layout::TextDef>,
}

/// HP bar lag effect state.
/// Tracks delayed HP percentage for smooth decrease animation.
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct HPBarLag {
    pub lag_hp_ratio: f32,
    pub last_hp_ratio: f32,
    pub start_lag_ratio: f32, // The ratio when the drain animation started
    pub delay_timer: f32,     // Wait before animation starts
    pub anim_progress: f32,   // 0.0 to 0.5 (seconds)
}

impl Default for HPBarLag {
    fn default() -> Self {
        Self {
            lag_hp_ratio: 1.0,
            last_hp_ratio: 1.0,
            start_lag_ratio: 1.0,
            delay_timer: 0.0,
            anim_progress: 0.0,
        }
    }
}

impl HPBarLag {
    pub fn new(hp_ratio: f32) -> Self {
        Self {
            lag_hp_ratio: hp_ratio,
            last_hp_ratio: hp_ratio,
            start_lag_ratio: hp_ratio,
            delay_timer: 0.0,
            anim_progress: 0.5, // Start finished
        }
    }
}

// ============================================================================
// View Element System - Phase 2 Refactoring
// 视图元素系统 - Phase 2 重构
// ============================================================================

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

// ============================================================================
// View Element History - for Undo/Redo/Reset functionality
// 视图元素历史 - 用于撤销/重做/重置功能
// ============================================================================

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
            current_index: -1,  // -1 means at original state
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
        // Clear redo stack when a new modification is made
        // 进行新修改时清除重做栈
        self.redo_stack.clear();
        
        // If we're in the middle of history (after undo), truncate future states
        // 如果我们在历史中间（撤销后），截断未来的状态
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
            // Already at original state, can't undo further
            // 已经在原始状态，无法进一步撤销
            return None;
        }
        
        // Save current state to redo stack
        // 将当前状态保存到重做栈
        let current = self.history[self.current_index as usize].clone();
        self.redo_stack.push(current);
        
        // Move to previous state
        // 移动到之前的状态
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
        (self.history.len(), self.redo_stack.len(), self.current_index)
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
            texture: None,  // We don't track texture path currently
        }
    }
}
