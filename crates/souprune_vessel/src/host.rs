//! Host-side SoupRune integration for Vessel output.
//!
//! Vessel 输出的宿主侧 SoupRune 集成。

use anyhow::Result;
use std::path::Path;

/// Build a Vessel component.
///
/// 构建 Vessel component。
pub fn build_component(
    component_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<vessel::BuildSummary> {
    vessel::build_component(component_path, output_dir)
}

/// Write generated files to the output directory.
///
/// 将生成文件写入输出目录。
pub fn write_generated_files(files: &[vessel::GeneratedRonFile], output_dir: &Path) -> Result<()> {
    vessel::write_generated_files(files, output_dir)
}
