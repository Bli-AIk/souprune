//! File output — writes RON strings to disk.
//!
//! 文件输出 — 将 RON 字符串写入磁盘。

use crate::component_host::GeneratedRonFile;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write generated files emitted by a Vessel content component.
///
/// 写入 Vessel 内容组件发射的生成文件。
pub fn write_generated_files(files: &[GeneratedRonFile], output_dir: &Path) -> Result<()> {
    for file in files {
        let full = output_dir.join(&file.path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }
        fs::write(&full, &file.ron_text)
            .with_context(|| format!("failed to write: {}", full.display()))?;
    }
    Ok(())
}
