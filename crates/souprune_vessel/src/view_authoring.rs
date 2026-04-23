//! Ergonomic authoring helpers for SoupRune view assets.
//!
//! SoupRune view 资源的人体工学编写辅助工具。

use std::collections::HashMap;

use souprune_schema::val::Val;
use souprune_schema::view::{
    ConditionalStyleDef, CoordinateSystem, DataRequirement, EasingDef, ImageDef, InitialFactValue,
    LagAnimationDef, MaterialAnimationsDef, MaterialDef, MaterialParamValue, RepeatDef,
    SerializableAlignItems, SerializableColor, SerializableJustifyContent,
    SerializablePositionType, SerializableTransform, SerializableVal, SerializableVec2,
    SerializableVec3, SpriteDef, StateSpriteConfig, StyleDef, TextAlignDef, TextAnchorDef, TextDef,
    UiFlexDirection, ViewBoxLogicDef, ViewFontDef, ViewLayout, ViewNodeDef, Visual,
};

/// Converts authoring values into a static-or-expression float.
///
/// 将编写侧数值转换为静态或表达式浮点值。
pub trait IntoFloatValue {
    /// Convert into a schema value.
    ///
    /// 转换为 Schema 值。
    fn into_float_value(self) -> Val<f32>;
}

impl IntoFloatValue for f32 {
    fn into_float_value(self) -> Val<f32> {
        Val::Static(self)
    }
}

impl IntoFloatValue for f64 {
    fn into_float_value(self) -> Val<f32> {
        Val::Static(self as f32)
    }
}

impl IntoFloatValue for i32 {
    fn into_float_value(self) -> Val<f32> {
        Val::Static(self as f32)
    }
}

impl IntoFloatValue for Val<f32> {
    fn into_float_value(self) -> Val<f32> {
        self
    }
}

impl IntoFloatValue for &str {
    fn into_float_value(self) -> Val<f32> {
        Val::Expr(self.to_owned())
    }
}

impl IntoFloatValue for String {
    fn into_float_value(self) -> Val<f32> {
        Val::Expr(self)
    }
}

/// Static float value.
///
/// 静态浮点值。
pub fn static_float(value: f32) -> Val<f32> {
    Val::Static(value)
}

/// Runtime expression value.
///
/// 运行时表达式值。
pub fn expression(value: impl Into<String>) -> Val<f32> {
    Val::Expr(value.into())
}

/// Two-dimensional float vector.
///
/// 二维浮点向量。
pub fn vector2(x: impl IntoFloatValue, y: impl IntoFloatValue) -> SerializableVec2 {
    (x.into_float_value(), y.into_float_value())
}

/// Three-dimensional float vector.
///
/// 三维浮点向量。
pub fn vector3(
    x: impl IntoFloatValue,
    y: impl IntoFloatValue,
    z: impl IntoFloatValue,
) -> SerializableVec3 {
    (
        x.into_float_value(),
        y.into_float_value(),
        z.into_float_value(),
    )
}

/// Four-channel color.
///
/// 四通道颜色。
pub fn color(
    red: impl IntoFloatValue,
    green: impl IntoFloatValue,
    blue: impl IntoFloatValue,
    alpha: impl IntoFloatValue,
) -> SerializableColor {
    (
        red.into_float_value(),
        green.into_float_value(),
        blue.into_float_value(),
        alpha.into_float_value(),
    )
}

/// Opaque white.
///
/// 不透明白色。
pub fn white() -> SerializableColor {
    color(1.0, 1.0, 1.0, 1.0)
}

/// Opaque red.
///
/// 不透明红色。
pub fn red() -> SerializableColor {
    color(1.0, 0.0, 0.0, 1.0)
}

/// Create a view layout from root nodes.
///
/// 从根节点创建 view layout。
pub fn view_layout(roots: impl Into<Vec<ViewNodeDef>>) -> ViewLayout {
    ViewLayout {
        roots: roots.into(),
        requires: Vec::new(),
        facts: None,
        world_space: false,
        coordinate_system: CoordinateSystem::Standard,
    }
}

/// Create a view node with schema defaults.
///
/// 使用 Schema 默认值创建 view 节点。
pub fn view_node(name: impl Into<String>) -> ViewNodeDef {
    ViewNodeDef {
        name: name.into(),
        tags: Vec::new(),
        style: StyleDef::default(),
        visible_when: None,
        background_color: None,
        border_color: None,
        image: None,
        sprite: None,
        state_sprite: None,
        texts: Vec::new(),
        view_box: None,
        children: Vec::new(),
        repeat: None,
    }
}

/// Create a sprite definition.
///
/// 创建 sprite 定义。
pub fn view_sprite(visual: impl Into<String>) -> SpriteDef {
    SpriteDef {
        visual: Visual(visual.into()),
        initial_state: None,
        color: None,
        flip_x: false,
        flip_y: false,
        transform: None,
        pivot: None,
        frame_duration: None,
        visible_when: None,
        material: None,
    }
}

/// Create a text definition.
///
/// 创建文本定义。
pub fn view_text(
    id: impl Into<String>,
    content: impl Into<String>,
    font: impl Into<ViewFontDef>,
) -> TextDef {
    TextDef {
        id: id.into(),
        content: Some(content.into()),
        font: font.into(),
        align: None,
        anchor: None,
        world_scale: vector2(1.0, 1.0),
        color: white(),
        transform: transform(),
        line_height: None,
        char_spacing: None,
        word_spacing: None,
        conditional_style: None,
        visible_when: None,
    }
}

/// Empty transform.
///
/// 空 transform。
pub fn transform() -> SerializableTransform {
    SerializableTransform {
        translation: None,
        rotation: None,
        scale: None,
    }
}

/// Create a view box definition.
///
/// 创建 view box 定义。
pub fn view_box(width: f32, height: f32) -> ViewBoxLogicDef {
    ViewBoxLogicDef {
        width,
        height,
        border_width: 0.0,
        offset: vector3(0.0, 0.0, 0.0),
        fill_shader: None,
        structure_file: None,
        fill_color: None,
    }
}

/// Create a repeat definition.
///
/// 创建 repeat 定义。
pub fn repeat(source: impl Into<String>) -> RepeatDef {
    RepeatDef {
        source: source.into(),
        limit: None,
        index_var: None,
        item_var: None,
    }
}

/// Create a material definition.
///
/// 创建 material 定义。
pub fn material(shader: impl Into<String>) -> MaterialDef {
    MaterialDef {
        shader: shader.into(),
        params: HashMap::new(),
        animations: None,
        texture: None,
    }
}

/// Create a lag animation definition.
///
/// 创建 lag animation 定义。
pub fn lag_animation(source: impl Into<String>, target: impl Into<String>) -> LagAnimationDef {
    LagAnimationDef {
        source: source.into(),
        target: target.into(),
        delay: 0.2,
        duration: 0.4,
        easing: EasingDef::Linear,
    }
}

/// Chainable authoring helpers for view layouts.
///
/// view layout 的链式编写辅助方法。
pub trait ViewLayoutAuthoring {
    /// Set world-space rendering.
    ///
    /// 设置 world-space 渲染。
    fn world_space(self, world_space: bool) -> Self;

    /// Set coordinate system.
    ///
    /// 设置坐标系。
    fn coordinate_system(self, coordinate_system: CoordinateSystem) -> Self;

    /// Add one file requirement.
    ///
    /// 添加一个文件依赖。
    fn require_file(self, path: impl Into<String>) -> Self;

    /// Add one interface requirement.
    ///
    /// 添加一个接口依赖。
    fn require_interface(
        self,
        interface: impl Into<String>,
        expects: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self;

    /// Set initial facts.
    ///
    /// 设置初始 facts。
    fn initial_facts(
        self,
        facts: impl IntoIterator<Item = (impl Into<String>, InitialFactValue)>,
    ) -> Self;
}

impl ViewLayoutAuthoring for ViewLayout {
    fn world_space(mut self, world_space: bool) -> Self {
        self.world_space = world_space;
        self
    }

    fn coordinate_system(mut self, coordinate_system: CoordinateSystem) -> Self {
        self.coordinate_system = coordinate_system;
        self
    }

    fn require_file(mut self, path: impl Into<String>) -> Self {
        self.requires.push(DataRequirement::File(path.into()));
        self
    }

    fn require_interface(
        mut self,
        interface: impl Into<String>,
        expects: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.requires.push(DataRequirement::Interface {
            interface: interface.into(),
            expects: expects.into_iter().map(Into::into).collect(),
        });
        self
    }

    fn initial_facts(
        mut self,
        facts: impl IntoIterator<Item = (impl Into<String>, InitialFactValue)>,
    ) -> Self {
        self.facts = Some(
            facts
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        );
        self
    }
}

/// Chainable authoring helpers for view nodes.
///
/// view 节点的链式编写辅助方法。
pub trait ViewNodeAuthoring {
    /// Add tags.
    ///
    /// 添加 tags。
    fn tags(self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self;

    /// Set visibility expression.
    ///
    /// 设置可见性表达式。
    fn visible_when(self, condition: impl Into<String>) -> Self;

    /// Set style.
    ///
    /// 设置 style。
    fn style(self, style: StyleDef) -> Self;

    /// Set background color.
    ///
    /// 设置背景色。
    fn background_color(self, color: SerializableColor) -> Self;

    /// Set border color.
    ///
    /// 设置边框色。
    fn border_color(self, color: SerializableColor) -> Self;

    /// Set image.
    ///
    /// 设置 image。
    fn image(self, image: ImageDef) -> Self;

    /// Set sprite.
    ///
    /// 设置 sprite。
    fn sprite(self, sprite: SpriteDef) -> Self;

    /// Set state sprite.
    ///
    /// 设置 state sprite。
    fn state_sprite(self, state_sprite: StateSpriteConfig) -> Self;

    /// Add texts.
    ///
    /// 添加文本。
    fn texts(self, texts: impl IntoIterator<Item = TextDef>) -> Self;

    /// Set view box.
    ///
    /// 设置 view box。
    fn view_box(self, view_box: ViewBoxLogicDef) -> Self;

    /// Add children.
    ///
    /// 添加子节点。
    fn children(self, children: impl IntoIterator<Item = ViewNodeDef>) -> Self;

    /// Set repeat.
    ///
    /// 设置 repeat。
    fn repeat(self, repeat: RepeatDef) -> Self;
}

impl ViewNodeAuthoring for ViewNodeDef {
    fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    fn visible_when(mut self, condition: impl Into<String>) -> Self {
        self.visible_when = Some(condition.into());
        self
    }

    fn style(mut self, style: StyleDef) -> Self {
        self.style = style;
        self
    }

    fn background_color(mut self, color: SerializableColor) -> Self {
        self.background_color = Some(color);
        self
    }

    fn border_color(mut self, color: SerializableColor) -> Self {
        self.border_color = Some(color);
        self
    }

    fn image(mut self, image: ImageDef) -> Self {
        self.image = Some(image);
        self
    }

    fn sprite(mut self, sprite: SpriteDef) -> Self {
        self.sprite = Some(sprite);
        self
    }

    fn state_sprite(mut self, state_sprite: StateSpriteConfig) -> Self {
        self.state_sprite = Some(state_sprite);
        self
    }

    fn texts(mut self, texts: impl IntoIterator<Item = TextDef>) -> Self {
        self.texts = texts.into_iter().collect();
        self
    }

    fn view_box(mut self, view_box: ViewBoxLogicDef) -> Self {
        self.view_box = Some(view_box);
        self
    }

    fn children(mut self, children: impl IntoIterator<Item = ViewNodeDef>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    fn repeat(mut self, repeat: RepeatDef) -> Self {
        self.repeat = Some(repeat);
        self
    }
}

/// Chainable authoring helpers for sprites.
///
/// sprite 的链式编写辅助方法。
pub trait SpriteAuthoring {
    /// Set initial state.
    ///
    /// 设置初始状态。
    fn initial_state(self, state: impl Into<String>) -> Self;

    /// Set color.
    ///
    /// 设置颜色。
    fn color(self, color: SerializableColor) -> Self;

    /// Set horizontal flip.
    ///
    /// 设置水平翻转。
    fn horizontal_flip(self, horizontal_flip: bool) -> Self;

    /// Set vertical flip.
    ///
    /// 设置垂直翻转。
    fn vertical_flip(self, vertical_flip: bool) -> Self;

    /// Set transform.
    ///
    /// 设置 transform。
    fn transform(self, transform: SerializableTransform) -> Self;

    /// Set translation.
    ///
    /// 设置平移。
    fn translation(self, translation: SerializableVec3) -> Self;

    /// Set rotation.
    ///
    /// 设置旋转。
    fn rotation(self, rotation: impl IntoFloatValue) -> Self;

    /// Set scale.
    ///
    /// 设置缩放。
    fn scale(self, scale: SerializableVec3) -> Self;

    /// Set pivot.
    ///
    /// 设置 pivot。
    fn pivot(self, pivot: SerializableVec2) -> Self;

    /// Set frame duration.
    ///
    /// 设置帧时长。
    fn frame_duration(self, frame_duration: f32) -> Self;

    /// Set visibility expression.
    ///
    /// 设置可见性表达式。
    fn visible_when(self, condition: impl Into<String>) -> Self;

    /// Set material.
    ///
    /// 设置 material。
    fn material(self, material: MaterialDef) -> Self;
}

impl SpriteAuthoring for SpriteDef {
    fn initial_state(mut self, state: impl Into<String>) -> Self {
        self.initial_state = Some(state.into());
        self
    }

    fn color(mut self, color: SerializableColor) -> Self {
        self.color = Some(color);
        self
    }

    fn horizontal_flip(mut self, horizontal_flip: bool) -> Self {
        self.flip_x = horizontal_flip;
        self
    }

    fn vertical_flip(mut self, vertical_flip: bool) -> Self {
        self.flip_y = vertical_flip;
        self
    }

    fn transform(mut self, transform: SerializableTransform) -> Self {
        self.transform = Some(transform);
        self
    }

    fn translation(mut self, translation: SerializableVec3) -> Self {
        self.transform = Some(
            self.transform
                .unwrap_or_else(transform)
                .translation(translation),
        );
        self
    }

    fn rotation(mut self, rotation: impl IntoFloatValue) -> Self {
        self.transform = Some(self.transform.unwrap_or_else(transform).rotation(rotation));
        self
    }

    fn scale(mut self, scale: SerializableVec3) -> Self {
        self.transform = Some(self.transform.unwrap_or_else(transform).scale(scale));
        self
    }

    fn pivot(mut self, pivot: SerializableVec2) -> Self {
        self.pivot = Some(pivot);
        self
    }

    fn frame_duration(mut self, frame_duration: f32) -> Self {
        self.frame_duration = Some(frame_duration);
        self
    }

    fn visible_when(mut self, condition: impl Into<String>) -> Self {
        self.visible_when = Some(condition.into());
        self
    }

    fn material(mut self, material: MaterialDef) -> Self {
        self.material = Some(material);
        self
    }
}

/// Chainable authoring helpers for text.
///
/// 文本的链式编写辅助方法。
pub trait TextAuthoring {
    /// Set text content.
    ///
    /// 设置文本内容。
    fn content(self, content: impl Into<String>) -> Self;

    /// Set text alignment.
    ///
    /// 设置文本对齐。
    fn align(self, align: TextAlignDef) -> Self;

    /// Set text anchor.
    ///
    /// 设置文本锚点。
    fn anchor(self, anchor: TextAnchorDef) -> Self;

    /// Set world scale.
    ///
    /// 设置 world scale。
    fn world_scale(self, scale: SerializableVec2) -> Self;

    /// Set color.
    ///
    /// 设置颜色。
    fn color(self, color: SerializableColor) -> Self;

    /// Set transform.
    ///
    /// 设置 transform。
    fn transform(self, transform: SerializableTransform) -> Self;

    /// Set translation.
    ///
    /// 设置平移。
    fn translation(self, translation: SerializableVec3) -> Self;

    /// Set rotation.
    ///
    /// 设置旋转。
    fn rotation(self, rotation: impl IntoFloatValue) -> Self;

    /// Set scale.
    ///
    /// 设置缩放。
    fn scale(self, scale: SerializableVec3) -> Self;

    /// Set line height.
    ///
    /// 设置行高。
    fn line_height(self, line_height: f32) -> Self;

    /// Set character spacing.
    ///
    /// 设置字符间距。
    fn character_spacing(self, character_spacing: f32) -> Self;

    /// Set word spacing.
    ///
    /// 设置词间距。
    fn word_spacing(self, word_spacing: f32) -> Self;

    /// Set conditional color.
    ///
    /// 设置条件颜色。
    fn conditional_color(self, condition: impl Into<String>, color: SerializableColor) -> Self;

    /// Set visibility expression.
    ///
    /// 设置可见性表达式。
    fn visible_when(self, condition: impl Into<String>) -> Self;
}

impl TextAuthoring for TextDef {
    fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    fn align(mut self, align: TextAlignDef) -> Self {
        self.align = Some(align);
        self
    }

    fn anchor(mut self, anchor: TextAnchorDef) -> Self {
        self.anchor = Some(anchor);
        self
    }

    fn world_scale(mut self, scale: SerializableVec2) -> Self {
        self.world_scale = scale;
        self
    }

    fn color(mut self, color: SerializableColor) -> Self {
        self.color = color;
        self
    }

    fn transform(mut self, transform: SerializableTransform) -> Self {
        self.transform = transform;
        self
    }

    fn translation(mut self, translation: SerializableVec3) -> Self {
        self.transform = self.transform.translation(translation);
        self
    }

    fn rotation(mut self, rotation: impl IntoFloatValue) -> Self {
        self.transform = self.transform.rotation(rotation);
        self
    }

    fn scale(mut self, scale: SerializableVec3) -> Self {
        self.transform = self.transform.scale(scale);
        self
    }

    fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    fn character_spacing(mut self, character_spacing: f32) -> Self {
        self.char_spacing = Some(character_spacing);
        self
    }

    fn word_spacing(mut self, word_spacing: f32) -> Self {
        self.word_spacing = Some(word_spacing);
        self
    }

    fn conditional_color(mut self, condition: impl Into<String>, color: SerializableColor) -> Self {
        self.conditional_style = Some(ConditionalStyleDef {
            condition: condition.into(),
            color,
        });
        self
    }

    fn visible_when(mut self, condition: impl Into<String>) -> Self {
        self.visible_when = Some(condition.into());
        self
    }
}

/// Chainable authoring helpers for transforms.
///
/// transform 的链式编写辅助方法。
pub trait TransformAuthoring {
    /// Set translation.
    ///
    /// 设置平移。
    fn translation(self, translation: SerializableVec3) -> Self;

    /// Set rotation.
    ///
    /// 设置旋转。
    fn rotation(self, rotation: impl IntoFloatValue) -> Self;

    /// Set scale.
    ///
    /// 设置缩放。
    fn scale(self, scale: SerializableVec3) -> Self;
}

impl TransformAuthoring for SerializableTransform {
    fn translation(mut self, translation: SerializableVec3) -> Self {
        self.translation = Some(translation);
        self
    }

    fn rotation(mut self, rotation: impl IntoFloatValue) -> Self {
        self.rotation = Some(rotation.into_float_value());
        self
    }

    fn scale(mut self, scale: SerializableVec3) -> Self {
        self.scale = Some(scale);
        self
    }
}

/// Chainable authoring helpers for view boxes.
///
/// view box 的链式编写辅助方法。
pub trait ViewBoxAuthoring {
    /// Set border width.
    ///
    /// 设置边框宽度。
    fn border_width(self, border_width: f32) -> Self;

    /// Set offset.
    ///
    /// 设置偏移。
    fn offset(self, offset: SerializableVec3) -> Self;

    /// Set fill shader.
    ///
    /// 设置填充 shader。
    fn fill_shader(self, shader: impl Into<String>) -> Self;

    /// Set structure file.
    ///
    /// 设置结构文件。
    fn structure_file(self, path: impl Into<String>) -> Self;

    /// Set fill color.
    ///
    /// 设置填充颜色。
    fn fill_color(self, color: SerializableColor) -> Self;
}

impl ViewBoxAuthoring for ViewBoxLogicDef {
    fn border_width(mut self, border_width: f32) -> Self {
        self.border_width = border_width;
        self
    }

    fn offset(mut self, offset: SerializableVec3) -> Self {
        self.offset = offset;
        self
    }

    fn fill_shader(mut self, shader: impl Into<String>) -> Self {
        self.fill_shader = Some(shader.into());
        self
    }

    fn structure_file(mut self, path: impl Into<String>) -> Self {
        self.structure_file = Some(path.into());
        self
    }

    fn fill_color(mut self, color: SerializableColor) -> Self {
        self.fill_color = Some(color);
        self
    }
}

/// Chainable authoring helpers for repeat definitions.
///
/// repeat 定义的链式编写辅助方法。
pub trait RepeatAuthoring {
    /// Set repeat limit.
    ///
    /// 设置 repeat 限制。
    fn limit(self, limit: usize) -> Self;

    /// Set index variable.
    ///
    /// 设置索引变量。
    fn index_variable(self, index_variable: impl Into<String>) -> Self;

    /// Set item variable.
    ///
    /// 设置元素变量。
    fn item_variable(self, item_variable: impl Into<String>) -> Self;
}

impl RepeatAuthoring for RepeatDef {
    fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    fn index_variable(mut self, index_variable: impl Into<String>) -> Self {
        self.index_var = Some(index_variable.into());
        self
    }

    fn item_variable(mut self, item_variable: impl Into<String>) -> Self {
        self.item_var = Some(item_variable.into());
        self
    }
}

/// Chainable authoring helpers for materials.
///
/// material 的链式编写辅助方法。
pub trait MaterialAuthoring {
    /// Add a static parameter.
    ///
    /// 添加静态参数。
    fn static_parameter(self, name: impl Into<String>, value: f32) -> Self;

    /// Add an expression parameter.
    ///
    /// 添加表达式参数。
    fn expression_parameter(self, name: impl Into<String>, value: impl Into<String>) -> Self;

    /// Set lag animation.
    ///
    /// 设置 lag animation。
    fn lag_animation(self, animation: LagAnimationDef) -> Self;

    /// Set texture.
    ///
    /// 设置 texture。
    fn texture(self, texture: impl Into<String>) -> Self;
}

impl MaterialAuthoring for MaterialDef {
    fn static_parameter(mut self, name: impl Into<String>, value: f32) -> Self {
        self.params
            .insert(name.into(), MaterialParamValue::Static(value));
        self
    }

    fn expression_parameter(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.params
            .insert(name.into(), MaterialParamValue::Expr(value.into()));
        self
    }

    fn lag_animation(mut self, animation: LagAnimationDef) -> Self {
        self.animations = Some(MaterialAnimationsDef {
            lag: Some(animation),
        });
        self
    }

    fn texture(mut self, texture: impl Into<String>) -> Self {
        self.texture = Some(texture.into());
        self
    }
}

/// Chainable authoring helpers for lag animations.
///
/// lag animation 的链式编写辅助方法。
pub trait LagAnimationAuthoring {
    /// Set delay.
    ///
    /// 设置延迟。
    fn delay(self, delay: f32) -> Self;

    /// Set duration.
    ///
    /// 设置时长。
    fn duration(self, duration: f32) -> Self;

    /// Set easing.
    ///
    /// 设置 easing。
    fn easing(self, easing: EasingDef) -> Self;
}

impl LagAnimationAuthoring for LagAnimationDef {
    fn delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        self
    }

    fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    fn easing(mut self, easing: EasingDef) -> Self {
        self.easing = easing;
        self
    }
}

/// Chainable authoring helpers for styles.
///
/// style 的链式编写辅助方法。
pub trait StyleAuthoring {
    /// Set width.
    ///
    /// 设置宽度。
    fn width(self, width: SerializableVal) -> Self;

    /// Set height.
    ///
    /// 设置高度。
    fn height(self, height: SerializableVal) -> Self;

    /// Set left offset.
    ///
    /// 设置左偏移。
    fn left(self, left: SerializableVal) -> Self;

    /// Set right offset.
    ///
    /// 设置右偏移。
    fn right(self, right: SerializableVal) -> Self;

    /// Set top offset.
    ///
    /// 设置上偏移。
    fn top(self, top: SerializableVal) -> Self;

    /// Set bottom offset.
    ///
    /// 设置下偏移。
    fn bottom(self, bottom: SerializableVal) -> Self;

    /// Set position type.
    ///
    /// 设置 position type。
    fn position_type(self, position_type: SerializablePositionType) -> Self;

    /// Set flex direction.
    ///
    /// 设置 flex direction。
    fn flex_direction(self, flex_direction: UiFlexDirection) -> Self;

    /// Set justify content.
    ///
    /// 设置 justify content。
    fn justify_content(self, justify_content: SerializableJustifyContent) -> Self;

    /// Set align items.
    ///
    /// 设置 align items。
    fn align_items(self, align_items: SerializableAlignItems) -> Self;
}

impl StyleAuthoring for StyleDef {
    fn width(mut self, width: SerializableVal) -> Self {
        self.width = Some(width);
        self
    }

    fn height(mut self, height: SerializableVal) -> Self {
        self.height = Some(height);
        self
    }

    fn left(mut self, left: SerializableVal) -> Self {
        self.left = Some(left);
        self
    }

    fn right(mut self, right: SerializableVal) -> Self {
        self.right = Some(right);
        self
    }

    fn top(mut self, top: SerializableVal) -> Self {
        self.top = Some(top);
        self
    }

    fn bottom(mut self, bottom: SerializableVal) -> Self {
        self.bottom = Some(bottom);
        self
    }

    fn position_type(mut self, position_type: SerializablePositionType) -> Self {
        self.position_type = Some(position_type);
        self
    }

    fn flex_direction(mut self, flex_direction: UiFlexDirection) -> Self {
        self.flex_direction = Some(flex_direction);
        self
    }

    fn justify_content(mut self, justify_content: SerializableJustifyContent) -> Self {
        self.justify_content = Some(justify_content);
        self
    }

    fn align_items(mut self, align_items: SerializableAlignItems) -> Self {
        self.align_items = Some(align_items);
        self
    }
}
