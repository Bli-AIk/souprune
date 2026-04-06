//! Core data types for danmaku and spawn pattern interfaces.
//!
//! 弹幕和生成模式接口的核心数据类型。

use crate::context::Vec2;
use crate::exports;

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
