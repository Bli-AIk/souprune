//! Host-side SoupRune integration for Cauld-ron output.
//!
//! Cauld-ron 输出的宿主侧 SoupRune 集成。

use anyhow::Result;
use std::path::Path;

/// Build a Cauld-ron component.
///
/// 构建 Cauld-ron component。
pub fn build_component(
    component_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<cauld_ron::BuildSummary> {
    cauld_ron::build_component(component_path, output_dir)
}

/// Write generated files to the output directory.
///
/// 将生成文件写入输出目录。
pub fn write_generated_files(files: &[cauld_ron::GeneratedRonFile], output_dir: &Path) -> Result<()> {
    cauld_ron::write_generated_files(files, output_dir)
}
