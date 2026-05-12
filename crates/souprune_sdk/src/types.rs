//! Core data types for input, danmaku, and spawn pattern interfaces.
//!
//! 输入、弹幕和生成模式接口的核心数据类型。

use crate::context::Vec2;
use crate::{Action, exports};

/// High-level input context identifier.
///
/// 高层输入上下文标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputContextId {
    Overworld,
    Battle,
    Dialogue,
    View,
    Custom(String),
}

/// Target receiving an input command.
///
/// 接收输入命令的目标。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputTarget {
    ActiveView,
    FreScope,
    Behavior(String),
}

/// Navigation direction used by input commands.
///
/// 输入命令使用的导航方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    #[doc(hidden)]
    pub fn from_wit(direction: exports::souprune::plugin::behavior::Direction) -> Self {
        use exports::souprune::plugin::behavior::Direction as WitDirection;
        match direction {
            WitDirection::Up => Self::Up,
            WitDirection::Down => Self::Down,
            WitDirection::Left => Self::Left,
            WitDirection::Right => Self::Right,
        }
    }
}

/// Semantic input command after raw actions are normalized.
///
/// 原始动作被标准化之后得到的语义输入命令。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputCommand {
    Navigate(Direction),
    Confirm,
    Cancel,
    Menu,
}

impl InputContextId {
    #[doc(hidden)]
    pub fn from_wit(context: &exports::souprune::plugin::behavior::InputContextId) -> Self {
        use exports::souprune::plugin::behavior::InputContextKind as WitInputContextKind;
        match context.kind {
            WitInputContextKind::Overworld => Self::Overworld,
            WitInputContextKind::Battle => Self::Battle,
            WitInputContextKind::Dialogue => Self::Dialogue,
            WitInputContextKind::View => Self::View,
            WitInputContextKind::Custom => {
                Self::Custom(context.custom_name.clone().unwrap_or_default())
            }
        }
    }
}

impl InputCommand {
    #[doc(hidden)]
    pub fn from_wit(command: exports::souprune::plugin::behavior::InputCommand) -> Self {
        use exports::souprune::plugin::behavior::InputCommand as WitInputCommand;
        match command {
            WitInputCommand::Navigate(direction) => Self::Navigate(Direction::from_wit(direction)),
            WitInputCommand::Confirm => Self::Confirm,
            WitInputCommand::Cancel => Self::Cancel,
            WitInputCommand::Menu => Self::Menu,
        }
    }
}

impl From<Action> for InputCommand {
    fn from(action: Action) -> Self {
        match action {
            Action::Up => Self::Navigate(Direction::Up),
            Action::Down => Self::Navigate(Direction::Down),
            Action::Left => Self::Navigate(Direction::Left),
            Action::Right => Self::Navigate(Direction::Right),
            Action::Confirm => Self::Confirm,
            Action::Cancel => Self::Cancel,
            Action::Menu => Self::Menu,
        }
    }
}

/// A lightweight input effect request emitted by input handlers.
///
/// 输入处理器发出的轻量输入效果请求。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputEffect {
    EmitEvent(String),
    OpenView(String),
    CloseView,
}

/// A single input transaction traveling through the routing layer.
///
/// 经过路由层流转的一笔输入事务。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputEnvelope {
    pub context: InputContextId,
    pub target: InputTarget,
    pub command: InputCommand,
    pub source_action: String,
}

impl InputEnvelope {
    /// Create a new input envelope.
    ///
    /// 创建一个新的输入事务封装。
    pub fn new(
        context: InputContextId,
        target: InputTarget,
        command: InputCommand,
        source_action: impl Into<String>,
    ) -> Self {
        Self {
            context,
            target,
            command,
            source_action: source_action.into(),
        }
    }
}

/// Result of handling an input transaction.
///
/// 处理输入事务后的结果。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputResult {
    Ignored,
    Consumed(Vec<InputEffect>),
    PassThrough(Vec<InputEffect>),
}

/// Bullet context for danmaku callbacks.
#[derive(Debug, Clone)]
pub struct BulletContext {
    pub elapsed: f32,
    pub delta_time: f32,
    pub spawn_pos: Vec2,
    pub offset: Vec2,
    pub initial_angle: f32,
    pub initial_radius: f32,
    pub player_pos: Vec2,
    props: Vec<(String, f32)>,
}

impl BulletContext {
    #[doc(hidden)]
    pub fn from_wit(ctx: &exports::souprune::plugin::danmaku::BulletContext) -> Self {
        Self {
            elapsed: ctx.elapsed,
            delta_time: ctx.delta_time,
            spawn_pos: Vec2::new(ctx.spawn_pos.x, ctx.spawn_pos.y),
            offset: Vec2::new(ctx.offset.x, ctx.offset.y),
            initial_angle: ctx.initial_angle,
            initial_radius: ctx.initial_radius,
            player_pos: Vec2::new(ctx.player_pos.x, ctx.player_pos.y),
            props: ctx
                .props
                .iter()
                .map(|p| (p.name.clone(), p.value))
                .collect(),
        }
    }

    /// Get the actual spawn position of this bullet (center + offset).
    pub fn spawn_position(&self) -> Vec2 {
        self.spawn_pos + self.offset
    }

    /// Get a named property value, or None if not found.
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.props.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
    }
}

/// Context for spawn pattern generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnContext {
    pub center: Vec2,
    pub player_pos: Vec2,
    pub time: f32,
}

impl SpawnContext {
    #[doc(hidden)]
    pub fn from_wit(ctx: &exports::souprune::plugin::spawn_pattern::SpawnContext) -> Self {
        Self {
            center: Vec2::new(ctx.center_x, ctx.center_y),
            player_pos: Vec2::new(ctx.player_x, ctx.player_y),
            time: ctx.time,
        }
    }
}

/// A computed spawn point output from a pattern.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnOutput {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub radius: f32,
}

impl SpawnOutput {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            angle: 0.0,
            radius: 0.0,
        }
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    #[doc(hidden)]
    pub fn to_wit(self) -> exports::souprune::plugin::spawn_pattern::SpawnPoint {
        exports::souprune::plugin::spawn_pattern::SpawnPoint {
            x: self.x,
            y: self.y,
            angle: self.angle,
            radius: self.radius,
        }
    }
}

/// Named parameter for spawn patterns.
#[derive(Debug, Clone)]
pub struct SpawnParam {
    pub name: String,
    pub value: f64,
}

impl SpawnParam {
    #[doc(hidden)]
    pub fn from_wit(p: &exports::souprune::plugin::spawn_pattern::PatternParam) -> Self {
        Self {
            name: p.name.clone(),
            value: p.value,
        }
    }

    /// Get as f32 for convenience.
    pub fn as_f32(&self) -> f32 {
        self.value as f32
    }

    /// Get as usize (clamped to 0).
    pub fn as_usize(&self) -> usize {
        self.value.max(0.0) as usize
    }
}

/// Output from a danmaku update.
#[derive(Debug, Clone, Copy, Default)]
pub struct BulletOutput {
    pub offset: Vec2,
    pub rotation: f32,
    /// Opacity override. Negative means no change.
    pub opacity: f32,
    /// Scale delta (0.0 = no change from base scale).
    pub scale_x: f32,
    pub scale_y: f32,
}

impl BulletOutput {
    pub const ZERO: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
        opacity: -1.0,
        scale_x: 0.0,
        scale_y: 0.0,
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            ..Self::ZERO
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_scale(mut self, sx: f32, sy: f32) -> Self {
        self.scale_x = sx;
        self.scale_y = sy;
        self
    }

    #[doc(hidden)]
    pub fn to_wit(self) -> exports::souprune::plugin::danmaku::BulletOutput {
        exports::souprune::plugin::danmaku::BulletOutput {
            offset: crate::souprune::plugin::host_api::Vec2 {
                x: self.offset.x,
                y: self.offset.y,
            },
            rotation: self.rotation,
            opacity: self.opacity,
            scale_x: self.scale_x,
            scale_y: self.scale_y,
        }
    }
}
