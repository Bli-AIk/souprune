/// Load UI solid fill shader body from external file.
pub fn load_ui_solid_fill_body() -> String {
    std::fs::read_to_string("projects/example/shaders/ui_solid_fill.wgsl").unwrap_or_else(|e| {
        eprintln!(
            "Warning: Failed to load shader from projects/example/shaders/ui_solid_fill.wgsl: {}",
            e
        );
        "let a = select(0.0, 1.0, input.distance <= 0.0);\nreturn vec4<f32>(input.color.rgb, a);"
            .to_string()
    })
}
