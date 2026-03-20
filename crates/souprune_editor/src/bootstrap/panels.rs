use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::panels;

pub(super) fn register_panels(app: &mut App) {
    app.register_panel(panels::AssetBrowserPanel::new());
    app.register_panel(panels::SequenceTimelinePanel::new());
    app.register_panel(panels::ChapterInspectorPanel::new());
    app.register_panel(panels::PlaybackPanel::new());
    app.register_panel(panels::FrePanel::new());
    app.register_panel(panels::ViewEditorPanel::new());
}
