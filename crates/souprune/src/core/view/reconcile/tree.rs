//! # tree.rs
//!
//! # 视图树数据结构
//!
//! Defines the DesiredViewTree and CurrentViewTree structures.
//!
//! 定义期望视图树和当前视图树结构。

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_bitmap_text::{TextAlign, TextAnchor};
use std::collections::HashMap;

use crate::core::view::layout::ViewLayoutRect;
use crate::core::view::layout::view_schema::EasingDef;

/// Unique identifier for a view element.
/// Used for matching between current and desired state.
///
/// 视图元素的唯一标识符。
/// 用于在当前状态和期望状态之间进行匹配。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViewElementKey {
    /// Full name including namespace, e.g., "battle::EnemyHpBar_0"
    /// 包含命名空间的完整名称
    pub full_name: String,

    /// Repeat index if this is a generated element from repeat
    /// 如果这是从 repeat 生成的元素，则为重复索引
    pub repeat_index: Option<usize>,
}

impl ViewElementKey {
    /// Create a new key for a non-repeat element.
    /// 为非重复元素创建新键。
    pub fn new(full_name: impl Into<String>) -> Self {
        Self {
            full_name: full_name.into(),
            repeat_index: None,
        }
    }

    /// Create a new key for a repeat-generated element.
    /// 为重复生成的元素创建新键。
    pub fn with_repeat_index(full_name: impl Into<String>, index: usize) -> Self {
        Self {
            full_name: full_name.into(),
            repeat_index: Some(index),
        }
    }
}

/// Desired state for a sprite component.
/// 精灵组件的期望状态。
#[derive(Clone, Debug, PartialEq)]
pub struct DesiredSprite {
    /// Texture path or procedural reference
    /// 纹理路径或程序化引用
    pub visual: String,

    /// Color tint
    /// 颜色着色
    pub color: Color,

    /// Anchor point
    /// 锚点
    pub anchor: Anchor,

    /// Flip flags
    /// 翻转标志
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Default for DesiredSprite {
    fn default() -> Self {
        Self {
            visual: String::new(),
            color: Color::WHITE,
            anchor: Anchor::CENTER,
            flip_x: false,
            flip_y: false,
        }
    }
}

/// Desired state for text component.
/// 文本组件的期望状态。
#[derive(Clone, Debug, PartialEq)]
pub struct DesiredText {
    /// Text content (may be template like "HP: {$hp}")
    /// 文本内容（可能是模板如 "HP: {$hp}"）
    pub content: String,

    /// Font identifier
    /// 字体标识符
    pub font: String,

    /// Font size
    /// 字体大小
    pub font_size: f32,

    /// Text color
    /// 文本颜色
    pub color: Color,

    /// Horizontal alignment
    /// 水平对齐
    pub align: TextAlign,

    /// Text anchor
    /// 文本锚点
    pub anchor: TextAnchor,
}

impl Default for DesiredText {
    fn default() -> Self {
        Self {
            content: String::new(),
            font: "default".into(),
            font_size: 16.0,
            color: Color::WHITE,
            align: TextAlign::Left,
            anchor: TextAnchor::CENTER,
        }
    }
}

/// Desired state for HP bar component.
/// HP 条组件的期望状态。
#[derive(Clone, Debug, PartialEq)]
pub struct DesiredHealthBar {
    /// HP source type (Player, Enemy{index}, Custom)
    /// HP 来源类型
    pub source_type: HealthSourceType,

    /// Current HP value (resolved)
    /// 当前 HP 值（已解析）
    pub current_hp: f32,

    /// Max HP value (resolved)
    /// 最大 HP 值（已解析）
    pub max_hp: f32,
}

/// Desired state for shader material component.
/// 着色器材质组件的期望状态。
#[derive(Clone, Debug, PartialEq)]
pub struct DesiredMaterial {
    /// Shader path
    /// 着色器路径
    pub shader: String,

    /// Parameter definitions (name -> expression or static value)
    /// 参数定义（名称 -> 表达式或静态值）
    pub params: std::collections::HashMap<String, MaterialParamDef>,

    /// Animation configuration
    /// 动画配置
    pub animations: Option<DesiredMaterialAnimations>,
}

/// Parameter definition for materials.
/// 材质的参数定义。
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialParamDef {
    Static(f32),
    Expr(String),
}

/// Animation configuration for materials.
/// 材质的动画配置。
#[derive(Clone, Debug, PartialEq)]
pub struct DesiredMaterialAnimations {
    /// Lag animation (source -> target param with easing)
    /// 延迟动画（源参数 -> 目标参数，带缓动）
    pub lag: Option<DesiredLagAnimation>,
}

/// Lag animation configuration.
/// 延迟动画配置。
#[derive(Clone, Debug, PartialEq)]
pub struct DesiredLagAnimation {
    pub source: String,
    pub target: String,
    pub delay: f32,
    pub duration: f32,
    pub easing: EasingDef,
}

/// HP source type for HP bars — uses FRE fact expressions for hp/hp_max.
/// HP 条的 HP 来源类型 — 通过 FRE fact 表达式获取 hp/hp_max。
#[derive(Clone, Debug, PartialEq)]
pub struct HealthSourceType {
    pub hp_expr: String,
    pub max_expr: String,
}

/// Desired state for a single view element.
/// 单个视图元素的期望状态。
#[derive(Clone, Debug)]
pub struct DesiredElement {
    /// Element key for matching
    /// 用于匹配的元素键
    pub key: ViewElementKey,

    /// Element name (without namespace)
    /// 元素名称（不含命名空间）
    pub name: String,

    /// Tags for categorization
    /// 用于分类的标签
    pub tags: Vec<String>,

    /// Transform (position, rotation, scale)
    /// 变换（位置、旋转、缩放）
    pub transform: Transform,

    /// Computed layout rectangle, if this element participates in layout.
    /// 如果此元素参与布局，则为计算后的布局矩形。
    pub layout_rect: Option<ViewLayoutRect>,

    /// Visibility state
    /// 可见性状态
    pub visibility: Visibility,

    /// Optional sprite definition
    /// 可选的精灵定义
    pub sprite: Option<DesiredSprite>,

    /// Optional text definitions
    /// 可选的文本定义
    pub texts: Vec<DesiredText>,

    /// Optional HP bar definition
    /// 可选的 HP 条定义
    pub health_bar: Option<DesiredHealthBar>,

    /// Optional material definition (for DynamicMaterial2d)
    /// 可选的材质定义（用于 DynamicMaterial2d）
    pub material: Option<DesiredMaterial>,

    /// Child elements
    /// 子元素
    pub children: Vec<DesiredElement>,

    /// visible_when expression (stored for component update)
    /// visible_when 表达式（存储用于组件更新）
    pub visible_when_expr: Option<String>,
}

impl DesiredElement {
    /// Create a new desired element with the given key.
    /// 使用给定的键创建新的期望元素。
    pub fn new(key: ViewElementKey, name: impl Into<String>) -> Self {
        Self {
            key,
            name: name.into(),
            tags: Vec::new(),
            transform: Transform::IDENTITY,
            layout_rect: None,
            visibility: Visibility::Inherited,
            sprite: None,
            texts: Vec::new(),
            health_bar: None,
            material: None,
            children: Vec::new(),
            visible_when_expr: None,
        }
    }

    /// Recursively collect all element keys in this subtree.
    /// 递归收集此子树中的所有元素键。
    pub fn collect_keys(&self) -> Vec<ViewElementKey> {
        let mut keys = vec![self.key.clone()];
        for child in &self.children {
            keys.extend(child.collect_keys());
        }
        keys
    }
}

/// The complete desired view tree.
/// 完整的期望视图树。
#[derive(Clone, Debug, Default)]
pub struct DesiredViewTree {
    /// Root elements
    /// 根元素
    pub roots: Vec<DesiredElement>,
}

impl DesiredViewTree {
    /// Create an empty desired view tree.
    /// 创建空的期望视图树。
    pub fn new() -> Self {
        Self { roots: Vec::new() }
    }

    /// Collect all element keys in the tree.
    /// 收集树中的所有元素键。
    pub fn collect_all_keys(&self) -> Vec<ViewElementKey> {
        self.roots.iter().flat_map(|r| r.collect_keys()).collect()
    }

    /// Find an element by key.
    /// 按键查找元素。
    pub fn find_by_key(&self, key: &ViewElementKey) -> Option<&DesiredElement> {
        fn find_in_element<'a>(
            element: &'a DesiredElement,
            key: &ViewElementKey,
        ) -> Option<&'a DesiredElement> {
            if &element.key == key {
                return Some(element);
            }
            element
                .children
                .iter()
                .find_map(|child| find_in_element(child, key))
        }

        for root in &self.roots {
            if let Some(found) = find_in_element(root, key) {
                return Some(found);
            }
        }
        None
    }
}

/// Current state of a single view element (from ECS).
/// 单个视图元素的当前状态（来自 ECS）。
#[derive(Clone, Debug)]
pub struct CurrentElement {
    /// Entity ID
    /// 实体 ID
    pub entity: Entity,

    /// Element key
    /// 元素键
    pub key: ViewElementKey,

    /// Current transform
    /// 当前变换
    pub transform: Transform,

    /// Current computed layout rectangle, if present on the entity.
    /// 当前实体上的计算布局矩形（如果存在）。
    pub layout_rect: Option<ViewLayoutRect>,

    /// Current visibility
    /// 当前可见性
    pub visibility: Visibility,

    /// Parent entity (if any)
    /// 父实体（如果有）
    pub parent: Option<Entity>,

    /// Current sprite properties (if entity has Sprite component)
    /// 当前精灵属性（如果实体有 Sprite 组件）
    pub sprite: Option<CurrentSprite>,

    /// Current visible_when expression (if entity has VisibleWhen component)
    /// 当前 visible_when 表达式（如果实体有 VisibleWhen 组件）
    pub visible_when_expr: Option<String>,

    /// Whether this element has ShaderMaterial component
    /// 此元素是否有 ShaderMaterial 组件
    pub has_shader_material: bool,
}

/// Current material properties from ECS.
/// 来自 ECS 的当前材质属性。
#[derive(Clone, Debug, PartialEq)]
pub struct CurrentMaterial {
    /// Shader path (from ShaderMaterial component)
    /// 着色器路径（来自 ShaderMaterial 组件）
    pub shader_path: String,
}

/// Current sprite properties from ECS.
/// 来自 ECS 的当前精灵属性。
#[derive(Clone, Debug, PartialEq)]
pub struct CurrentSprite {
    /// Color tint
    /// 颜色着色
    pub color: Color,

    /// Flip flags
    /// 翻转标志
    pub flip_x: bool,
    pub flip_y: bool,
}

/// Current view tree state built from ECS queries.
/// 从 ECS 查询构建的当前视图树状态。
#[derive(Clone, Debug, Default)]
pub struct CurrentViewTree {
    /// All elements indexed by key
    /// 按键索引的所有元素
    pub elements: HashMap<ViewElementKey, CurrentElement>,
}

impl CurrentViewTree {
    /// Create an empty current view tree.
    /// 创建空的当前视图树。
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    /// Add an element to the tree.
    /// 向树中添加元素。
    pub fn insert(&mut self, element: CurrentElement) {
        self.elements.insert(element.key.clone(), element);
    }

    /// Get an element by key.
    /// 按键获取元素。
    pub fn get(&self, key: &ViewElementKey) -> Option<&CurrentElement> {
        self.elements.get(key)
    }

    /// Check if the tree contains an element with the given key.
    /// 检查树是否包含具有给定键的元素。
    pub fn contains(&self, key: &ViewElementKey) -> bool {
        self.elements.contains_key(key)
    }
}
