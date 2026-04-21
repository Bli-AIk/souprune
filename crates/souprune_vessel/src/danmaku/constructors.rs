//! Convenience constructors for danmaku schema types.
//!
//! 弹幕 Schema 类型的便利构造器。
//!
//! These functions provide a concise way to build `BulletBehavior`,
//! `ColliderShape`, `SpawnPattern`, and `TimelineEvent` values without
//! manually specifying every field.
//!
//! 这些函数提供了一种简洁的方式来构建弹幕相关类型，
//! 无需手动指定每个字段。

use souprune_schema::danmaku::*;

// ── BulletBehavior constructors ─────────────────────────────────────────

/// Create an `Aimed` behavior with the given speed.
///
/// 创建指定速度的自机狙行为。
pub fn aimed(speed: f32) -> BulletBehavior {
    BulletBehavior::Aimed(AimedConfig {
        speed,
        ..Default::default()
    })
}

/// Create a `Tween` behavior targeting a given property.
///
/// 创建针对指定属性的补间行为。
pub fn tween(target: DanmakuTweenTarget, duration: f32, ease: Easing, to: f32) -> BulletBehavior {
    BulletBehavior::Tween(TweenConfig {
        target,
        duration,
        ease,
        to,
        from: 0.0,
        delay: 0.0,
        mode: TweenMode::default(),
    })
}

/// Create a `Tween` behavior with a delay before it starts.
///
/// 创建带延迟的补间行为。
pub fn tween_delayed(
    target: DanmakuTweenTarget,
    duration: f32,
    ease: Easing,
    to: f32,
    delay: f32,
) -> BulletBehavior {
    BulletBehavior::Tween(TweenConfig {
        target,
        duration,
        ease,
        to,
        from: 0.0,
        delay,
        mode: TweenMode::default(),
    })
}

/// Create a `Linear` behavior with direction and speed.
///
/// 创建指定方向和速度的线性行为。
pub fn linear(dir: (f32, f32), speed: f32) -> BulletBehavior {
    BulletBehavior::Linear(LinearConfig { dir, speed })
}

// ── ColliderShape constructors ──────────────────────────────────────────

/// Create a circular collider.
///
/// 创建圆形碰撞体。
pub fn circle(radius: f32) -> ColliderShape {
    ColliderShape::CircleCollider(radius)
}

/// Create a rectangular collider.
///
/// 创建矩形碰撞体。
pub fn rect(w: f32, h: f32) -> ColliderShape {
    ColliderShape::BoxCollider(w, h)
}

// ── SpawnPattern constructors ───────────────────────────────────────────

/// Create a `BoxEdgeGenerator` pattern.
///
/// 创建基于 ViewBox 边缘的生成模式。
pub fn box_edge(box_name: &str, side: EdgeSide, count: usize) -> BoxEdgeBuilder {
    BoxEdgeBuilder {
        box_name: box_name.to_string(),
        count,
        side,
        spacing: 30.0,
        outside_margin: 0.0,
        randomness: 0.0,
    }
}

/// Builder for `BoxEdgeGenerator` spawn patterns.
///
/// `BoxEdgeGenerator` 生成模式的构建器。
pub struct BoxEdgeBuilder {
    box_name: String,
    count: usize,
    side: EdgeSide,
    spacing: f32,
    outside_margin: f32,
    randomness: f32,
}

impl BoxEdgeBuilder {
    pub fn spacing(mut self, s: f32) -> Self {
        self.spacing = s;
        self
    }

    /// Set distance outside the box edge.
    pub fn outside(mut self, m: f32) -> Self {
        self.outside_margin = m;
        self
    }

    pub fn randomness(mut self, r: f32) -> Self {
        self.randomness = r;
        self
    }

    pub fn build(self) -> SpawnPattern {
        SpawnPattern::BoxEdgeGenerator {
            box_name: self.box_name,
            count: self.count,
            side: self.side,
            spacing: self.spacing,
            outside_margin: self.outside_margin,
            randomness: self.randomness,
        }
    }
}

/// Create an `EdgeGenerator` pattern.
///
/// 创建屏幕边缘生成模式。
pub fn edge(side: EdgeSide, count: usize) -> EdgeBuilder {
    EdgeBuilder {
        count,
        side,
        spacing: 30.0,
        margin: 200.0,
        randomness: 0.0,
    }
}

/// Builder for `EdgeGenerator` spawn patterns.
pub struct EdgeBuilder {
    count: usize,
    side: EdgeSide,
    spacing: f32,
    margin: f32,
    randomness: f32,
}

impl EdgeBuilder {
    pub fn spacing(mut self, s: f32) -> Self {
        self.spacing = s;
        self
    }

    pub fn margin(mut self, m: f32) -> Self {
        self.margin = m;
        self
    }

    pub fn randomness(mut self, r: f32) -> Self {
        self.randomness = r;
        self
    }

    pub fn build(self) -> SpawnPattern {
        SpawnPattern::EdgeGenerator {
            count: self.count,
            side: self.side,
            spacing: self.spacing,
            margin: self.margin,
            randomness: self.randomness,
        }
    }
}

/// Create a `RingGenerator` pattern.
///
/// 创建环形生成模式。
pub fn ring(count: usize, radius: f32) -> RingBuilder {
    RingBuilder {
        count,
        radius,
        start_angle: 0.0,
        randomness: 0.0,
    }
}

/// Builder for `RingGenerator` spawn patterns.
pub struct RingBuilder {
    count: usize,
    radius: f32,
    start_angle: f32,
    randomness: f32,
}

impl RingBuilder {
    pub fn start_angle(mut self, a: f32) -> Self {
        self.start_angle = a;
        self
    }

    pub fn randomness(mut self, r: f32) -> Self {
        self.randomness = r;
        self
    }

    pub fn build(self) -> SpawnPattern {
        SpawnPattern::RingGenerator {
            count: self.count,
            radius: self.radius,
            start_angle: self.start_angle,
            randomness: self.randomness,
        }
    }
}

// ── TimelineEvent constructor ───────────────────────────────────────────

/// Create a timeline event at absolute time `t`.
///
/// 创建绝对时间 `t` 处的时间线事件。
pub fn event_at(t: f32, spawn: &str, pattern: SpawnPattern) -> TimelineEventBuilder {
    TimelineEventBuilder {
        t,
        time_mode: TimeMode::Absolute,
        spawn: spawn.to_string(),
        pattern,
        offset: (0.0, 0.0),
        apply: Vec::new(),
        behaviors: Vec::new(),
    }
}

/// Create a timeline event with delta time `dt` from the previous event.
///
/// 创建距前一事件 `dt` 秒的时间线事件。
pub fn event_delta(dt: f32, spawn: &str, pattern: SpawnPattern) -> TimelineEventBuilder {
    TimelineEventBuilder {
        t: dt,
        time_mode: TimeMode::Delta,
        spawn: spawn.to_string(),
        pattern,
        offset: (0.0, 0.0),
        apply: Vec::new(),
        behaviors: Vec::new(),
    }
}

/// Builder for `TimelineEvent`.
pub struct TimelineEventBuilder {
    t: f32,
    time_mode: TimeMode,
    spawn: String,
    pattern: SpawnPattern,
    offset: (f32, f32),
    apply: Vec<String>,
    behaviors: Vec<BulletBehavior>,
}

impl TimelineEventBuilder {
    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = (x, y);
        self
    }

    pub fn apply(mut self, names: &[&str]) -> Self {
        self.apply = names.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn behaviors(mut self, b: Vec<BulletBehavior>) -> Self {
        self.behaviors = b;
        self
    }

    pub fn build(self) -> TimelineEvent {
        TimelineEvent {
            t: self.t,
            time_mode: self.time_mode,
            spawn: self.spawn,
            pattern: self.pattern,
            offset: self.offset,
            apply: self.apply,
            behaviors: self.behaviors,
        }
    }
}

// ── BulletPrototype builder ─────────────────────────────────────────────

/// Start building a `BulletPrototype` with a visual asset path.
///
/// 以视觉资产路径开始构建弹幕原型。
pub fn prototype(visual: &str) -> PrototypeBuilder {
    PrototypeBuilder {
        inner: BulletPrototype {
            visual: visual.to_string(),
            ..Default::default()
        },
    }
}

/// Builder for `BulletPrototype`.
pub struct PrototypeBuilder {
    inner: BulletPrototype,
}

impl PrototypeBuilder {
    pub fn collider(mut self, c: ColliderShape) -> Self {
        self.inner.collider = c;
        self
    }

    pub fn damage(mut self, d: f32) -> Self {
        self.inner.damage = d;
        self
    }

    pub fn lifetime(mut self, l: f32) -> Self {
        self.inner.lifetime = l;
        self
    }

    pub fn z_index(mut self, z: f32) -> Self {
        self.inner.z_index = z;
        self
    }

    pub fn scale(mut self, s: f32) -> Self {
        self.inner.scale = s;
        self
    }

    pub fn frame_duration(mut self, d: f32) -> Self {
        self.inner.frame_duration = Some(d);
        self
    }

    pub fn build(self) -> BulletPrototype {
        self.inner
    }
}
