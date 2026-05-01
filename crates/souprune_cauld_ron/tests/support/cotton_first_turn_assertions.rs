//! Assertion helpers for the `cotton_first_turn` performance tests.
//!
//! `cotton_first_turn` 演出测试的断言辅助。

use souprune_schema::danmaku::DanmakuPerformance;

/// Parse generated RON text after stripping comment headers.
///
/// 去掉注释文件头后解析生成出的 RON 文本。
pub fn parse_generated_ron(ron_text: &str) -> DanmakuPerformance {
    let body = ron_text
        .lines()
        .filter(|line| !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    ron::from_str(&body).expect("generated RON should parse into DanmakuPerformance")
}

/// Assert that two performances match on the fields that matter for runtime behavior.
///
/// 断言两个演出在运行时关键字段上完全一致。
pub fn assert_same_performance(generated: &DanmakuPerformance, expected: &DanmakuPerformance) {
    assert_eq!(
        generated.prototypes.len(),
        expected.prototypes.len(),
        "prototype count mismatch"
    );
    for (name, generated_prototype) in &generated.prototypes {
        let expected_prototype = expected
            .prototypes
            .get(name)
            .unwrap_or_else(|| panic!("missing prototype: {name}"));
        assert_eq!(
            generated_prototype.visual, expected_prototype.visual,
            "visual mismatch: {name}"
        );
        assert!(
            (generated_prototype.damage - expected_prototype.damage).abs() < 0.01,
            "damage mismatch: {name}"
        );
        assert!(
            (generated_prototype.lifetime - expected_prototype.lifetime).abs() < 0.01,
            "lifetime mismatch: {name}"
        );
        assert!(
            (generated_prototype.z_index - expected_prototype.z_index).abs() < 0.01,
            "z_index mismatch: {name}"
        );
    }

    assert_eq!(
        generated.behaviors.len(),
        expected.behaviors.len(),
        "behavior count mismatch: generated={}, expected={}",
        generated.behaviors.len(),
        expected.behaviors.len()
    );

    assert_eq!(
        generated.timeline.len(),
        expected.timeline.len(),
        "timeline event count mismatch"
    );

    for (index, (generated_event, expected_event)) in generated
        .timeline
        .iter()
        .zip(expected.timeline.iter())
        .enumerate()
    {
        assert!(
            (generated_event.t - expected_event.t).abs() < 0.02,
            "timeline[{index}] time mismatch: generated={}, expected={}",
            generated_event.t,
            expected_event.t
        );
        assert_eq!(
            generated_event.spawn, expected_event.spawn,
            "timeline[{index}] spawn mismatch"
        );
    }

    let generated_duration = generated.resolved_duration();
    let expected_duration = expected.resolved_duration();
    assert!(
        generated_duration.is_some() && expected_duration.is_some(),
        "duration should exist in both performances"
    );
    assert!(
        (generated_duration.expect("generated duration")
            - expected_duration.expect("expected duration"))
        .abs()
            < 0.1,
        "duration mismatch: generated={generated_duration:?}, expected={expected_duration:?}"
    );
}
