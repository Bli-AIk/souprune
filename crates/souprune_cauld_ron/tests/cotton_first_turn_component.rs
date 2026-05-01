//! End-to-end test for `cotton_first_turn` through a real Cauld-ron guest component.
//!
//! `cotton_first_turn` 通过真实 Cauld-ron guest component 的端到端测试。

#[path = "support/cotton_first_turn_assertions.rs"]
mod cotton_first_turn_assertions;

use cotton_first_turn_assertions::{assert_same_performance, parse_generated_ron};
use souprune_schema::danmaku::DanmakuPerformance;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn cotton_first_turn_component_build_matches_reference_asset() {
    let fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../projects/mad_dummy_example/content");
    if !fixture_dir.join("Cargo.toml").exists() {
        eprintln!(
            "skipping local mad_dummy_example content component test; project submodule is not checked out"
        );
        return;
    }

    let target_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/content-wasm-tests");

    let build_status = Command::new("cargo")
        .current_dir(&fixture_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args(["build", "--target", "wasm32-wasip2"])
        .status()
        .expect("failed to invoke cargo build for mad_dummy_example content guest");
    assert!(
        build_status.success(),
        "mad_dummy_example content guest should build successfully"
    );

    let component_path = target_dir.join("wasm32-wasip2/debug/content.wasm");
    assert!(
        component_path.exists(),
        "mad_dummy_example content component should exist"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let output_dir = std::env::temp_dir().join(format!("cauld_ron_cotton_first_turn_{unique}"));
    let _ = fs::remove_dir_all(&output_dir);

    let summary = cauld_ron::build_component(&component_path, &output_dir)
        .expect("cauld-ron host should build cotton_first_turn from the guest component");
    assert_eq!(
        summary.written_files, 24,
        "expected all mad_dummy_example managed RON assets to be generated"
    );

    let output_path = output_dir.join("battle/danmaku/cotton_first_turn.performance.ron");
    assert!(
        output_path.exists(),
        "generated cotton_first_turn output should exist"
    );

    let generated_ron = fs::read_to_string(&output_path)
        .expect("generated cotton_first_turn output should be readable");
    assert!(
        generated_ron.contains("BOOTSTRAPPED BY CAULD-RON"),
        "generated output should contain the Cauld-ron bootstrap header"
    );
    assert!(
        generated_ron.contains("// Generated at: "),
        "generated output should contain a human-readable generation timestamp"
    );

    let generated = parse_generated_ron(&generated_ron);
    let original_ron = include_str!(
        "fixtures/project_ron_baselines/mad_dummy_example/battle/danmaku/cotton_first_turn.performance.ron"
    );
    let original: DanmakuPerformance =
        ron::from_str(original_ron).expect("reference cotton_first_turn should parse");

    assert_same_performance(&generated, &original);

    let _ = fs::remove_dir_all(&output_dir);
}
