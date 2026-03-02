//! 编辑器面板。

mod asset_browser;
mod chapter_inspector;
mod playback;
pub(crate) mod sequence_timeline;

pub use asset_browser::AssetBrowserPanel;
pub use chapter_inspector::ChapterInspectorPanel;
pub use playback::PlaybackPanel;
pub use sequence_timeline::SequenceTimelinePanel;
