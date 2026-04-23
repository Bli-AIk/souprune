//! Regression tests for explicit danmaku performance authoring.
//!
//! 显式弹幕演出编写方式的回归测试。

use souprune_schema::danmaku::{
    BulletBehavior, BulletPrototype, DanmakuPerformance, SpawnPattern, TimeMode, TimelineEvent,
};
use std::collections::HashMap;

#[test]
fn explicit_performance_authoring_supports_composed_parts() {
    let mut prototypes = HashMap::from([(
        "orb".to_string(),
        BulletPrototype::new("battle/bullets/orb.png"),
    )]);
    prototypes.extend([(
        "alt".to_string(),
        BulletPrototype::new("battle/bullets/alt.png"),
    )]);

    let mut behaviors = HashMap::from([("idle".to_string(), BulletBehavior::stationary())]);
    behaviors.extend([
        ("orbit".to_string(), BulletBehavior::orbital(1.5, 8.0)),
        (
            "wiggle".to_string(),
            BulletBehavior::sine((1.0, 0.0), 18.0, 2.5),
        ),
    ]);

    let mut timeline = vec![TimelineEvent::absolute(
        0.0,
        "orb",
        SpawnPattern::line(4, 16.0, (0.0, 1.0)),
        ["idle"],
    )];
    timeline.extend([TimelineEvent::delta_with(
        0.5,
        "orb",
        SpawnPattern::custom("example.spiral", [("turns", 3.0), ("radius", 24.0)]),
        (0.0, 0.0),
        Vec::<String>::new(),
        [BulletBehavior::stationary()],
    )]);

    let performance = DanmakuPerformance {
        prototypes,
        behaviors,
        timeline,
        duration: Some(3.0),
    };

    assert_eq!(performance.prototypes.len(), 2);
    assert_eq!(performance.behaviors.len(), 3);
    assert_eq!(performance.timeline.len(), 2);
    assert_eq!(performance.duration, Some(3.0));

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
