//! Regression tests for `performance!` and the Rust reference constructors.
//!
//! `performance!` 与 Rust 参考构造器的回归测试。

use souprune_schema::danmaku::{BulletBehavior, DurationExpr, SpawnPattern, TimeMode};
use souprune_vessel::prelude::*;
use std::collections::HashMap;

#[test]
fn performance_macro_supports_spreads_and_extended_constructors() {
    let extra_prototypes = HashMap::from([(
        "alt".to_string(),
        prototype("battle/bullets/alt.png").build(),
    )]);
    let extra_behaviors = HashMap::from([
        ("orbit".to_string(), orbital(1.5, 8.0)),
        ("wiggle".to_string(), sine((1.0, 0.0), 18.0, 2.5)),
    ]);
    let extra_timeline = vec![
        event_delta(
            0.5,
            "orb",
            custom_pattern("example.spiral")
                .param("turns", 3.0)
                .param("radius", 24.0)
                .build(),
        )
        .behaviors(vec![stationary()])
        .build(),
    ];

    let performance = performance! {
        prototypes {
            "orb" => prototype("battle/bullets/orb.png").build(),
            ..extra_prototypes,
        }
        behaviors {
            "idle" => stationary(),
            ..extra_behaviors,
        }
        timeline [
            event_at(0.0, "orb", line(4).spacing(16.0).direction((0.0, 1.0)).build())
                .apply(&["idle"])
                .build(),
            ..extra_timeline,
        ]
        duration: DurationExpr::Literal(3.0),
    };

    assert_eq!(performance.prototypes.len(), 2);
    assert_eq!(performance.behaviors.len(), 3);
    assert_eq!(performance.timeline.len(), 2);
    assert!(matches!(
        performance.duration,
        Some(DurationExpr::Literal(3.0))
    ));

    assert!(matches!(
        performance.behaviors.get("idle"),
        Some(BulletBehavior::Stationary())
    ));
    assert!(matches!(
        performance.behaviors.get("orbit"),
        Some(BulletBehavior::Orbital(_))
    ));
    assert!(matches!(
        performance.behaviors.get("wiggle"),
        Some(BulletBehavior::Sine(_))
    ));

    match &performance.timeline[0].pattern {
        SpawnPattern::LineGenerator {
            count,
            spacing,
            direction,
            randomness,
        } => {
            assert_eq!(*count, 4);
            assert_eq!(*spacing, 16.0);
            assert_eq!(*direction, (0.0, 1.0));
            assert_eq!(*randomness, 0.0);
        }
        other => panic!("expected line pattern, got {other:?}"),
    }
    assert!(matches!(
        performance.timeline[0].time_mode,
        TimeMode::Absolute
    ));
    assert_eq!(performance.timeline[0].apply, ["idle"]);

    match &performance.timeline[1].pattern {
        SpawnPattern::CustomGenerator { id, params } => {
            assert_eq!(id, "example.spiral");
            assert_eq!(params.get("turns"), Some(&3.0));
            assert_eq!(params.get("radius"), Some(&24.0));
        }
        other => panic!("expected custom pattern, got {other:?}"),
    }
    assert!(matches!(performance.timeline[1].time_mode, TimeMode::Delta));
    assert!(matches!(
        performance.timeline[1].behaviors.as_slice(),
        [BulletBehavior::Stationary()]
    ));
}
