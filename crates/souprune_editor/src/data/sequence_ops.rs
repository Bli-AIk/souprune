//! 序列文件 CRUD 操作。

use std::path::PathBuf;

use bevy::prelude::*;
use souprune::core::sequencer::chapter_schema::Chapter;
use souprune::core::sequencer::SequenceAsset;

/// 编辑器内部的序列表示，不依赖 Bevy Asset 系统。
#[derive(Debug, Clone)]
pub struct EditorSequence {
    /// 序列文件路径。
    pub file_path: PathBuf,
    /// 序列模式。
    pub mode: Option<String>,
    /// FRE 规则文件路径。
    pub rules_file: Option<String>,
    /// 命名退出点。
    pub exits: std::collections::HashMap<String, String>,
    /// 章节列表。
    pub chapters: Vec<Chapter>,
    /// 是否已修改（脏标记）。
    pub dirty: bool,
}

impl EditorSequence {
    pub fn from_asset(asset: &SequenceAsset, file_path: PathBuf) -> Self {
        Self {
            file_path,
            mode: asset.mode.clone(),
            rules_file: asset.rules_file.clone(),
            exits: asset.exits.clone(),
            chapters: asset.chapters.clone(),
            dirty: false,
        }
    }

    pub fn to_asset(&self) -> SequenceAsset {
        SequenceAsset {
            mode: self.mode.clone(),
            rules_file: self.rules_file.clone(),
            exits: self.exits.clone(),
            chapters: self.chapters.clone(),
        }
    }

    /// 插入章节到指定位置之后。
    pub fn insert_chapter(&mut self, index: usize, chapter: Chapter) {
        let pos = (index + 1).min(self.chapters.len());
        self.chapters.insert(pos, chapter);
        self.dirty = true;
    }

    /// 删除指定位置的章节。
    pub fn remove_chapter(&mut self, index: usize) -> Option<Chapter> {
        if index < self.chapters.len() {
            self.dirty = true;
            Some(self.chapters.remove(index))
        } else {
            None
        }
    }

    /// 移动章节（从 from 到 to）。
    #[allow(dead_code)]
    pub fn move_chapter(&mut self, from: usize, to: usize) {
        if from < self.chapters.len() && to < self.chapters.len() && from != to {
            let chapter = self.chapters.remove(from);
            self.chapters.insert(to, chapter);
            self.dirty = true;
        }
    }

    /// 复制指定章节并插入到其后。
    #[allow(dead_code)]
    pub fn duplicate_chapter(&mut self, index: usize) {
        if index < self.chapters.len() {
            let cloned = self.chapters[index].clone();
            self.chapters.insert(index + 1, cloned);
            self.dirty = true;
        }
    }
}

/// 序列文件操作事件。
#[derive(Event)]
#[allow(dead_code)]
pub enum SequenceFileEvent {
    /// 打开序列文件。
    Open(PathBuf),
    /// 保存当前序列。
    Save,
    /// 另存为。
    SaveAs(PathBuf),
}

/// 从文件路径加载序列（直接读取 RON，不经过 Bevy AssetServer）。
pub fn load_sequence_from_file(path: &std::path::Path) -> Result<EditorSequence, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {e}"))?;
    let asset: SequenceAsset =
        ron::from_str(&content).map_err(|e| format!("RON 解析失败: {e}"))?;
    Ok(EditorSequence::from_asset(&asset, path.to_path_buf()))
}

/// 将序列保存到文件。
pub fn save_sequence_to_file(seq: &EditorSequence) -> Result<(), String> {
    // 写入前创建 .bak 备份
    let bak_path = seq.file_path.with_extension("ron.bak");
    if seq.file_path.exists() {
        let _ = std::fs::copy(&seq.file_path, &bak_path);
    }

    let asset = seq.to_asset();
    let config = ron::ser::PrettyConfig::default()
        .struct_names(true)
        .enumerate_arrays(false);
    let content =
        ron::ser::to_string_pretty(&asset, config).map_err(|e| format!("RON 序列化失败: {e}"))?;
    std::fs::write(&seq.file_path, content).map_err(|e| format!("写入文件失败: {e}"))?;
    Ok(())
}

/// Debounced 自动保存系统：倒计时到 0 后触发保存。
pub fn auto_save_system(
    time: Res<Time>,
    mut state: ResMut<crate::panels::sequence_timeline::EditorSequenceState>,
) {
    let Some(timer) = state.save_timer.as_mut() else {
        return;
    };
    *timer -= time.delta_secs();
    if *timer <= 0.0 {
        state.save_timer = None;
        if let Some(seq) = state.current.as_mut()
            && seq.dirty
        {
            match save_sequence_to_file(seq) {
                Ok(()) => {
                    seq.dirty = false;
                    info!("序列已自动保存: {:?}", seq.file_path);
                }
                Err(e) => {
                    warn!("自动保存失败: {e}");
                }
            }
        }
    }
}
