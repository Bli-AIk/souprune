//! # Chapter Inspector Panel
//!
//! Chapter inspector panel for the editor.

use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune_schema::sequence::Chapter;

use super::sequence_timeline::EditorSequenceState;
use crate::data::ModifyChapterAction;
use crate::i18n::t;
use crate::widgets::property_editors::chapter_type_label;

mod editors;

use self::editors::render_chapter_properties;

/// 章节属性检查器面板。
pub struct ChapterInspectorPanel {
    cached_title: String,
}

impl ChapterInspectorPanel {
    pub fn new() -> Self {
        Self {
            cached_title: "Chapter Inspector".to_string(),
        }
    }
}

impl WorkbenchPanel for ChapterInspectorPanel {
    fn id(&self) -> &str {
        "chapter_inspector"
    }

    fn title(&self) -> String {
        self.cached_title.clone()
    }

    fn closable(&self) -> bool {
        true
    }

    fn default_visible(&self) -> bool {
        true
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        self.cached_title = t(world, "panel-chapter-inspector");

        if !world.contains_resource::<EditorSequenceState>() {
            ui.label(t(world, "label-not-initialized"));
            return;
        }

        let state = world.resource::<EditorSequenceState>();
        let Some(seq) = &state.current else {
            ui.label(t(world, "label-no-sequence-open"));
            return;
        };
        let Some(idx) = state.selected_chapter else {
            ui.centered_and_justified(|ui| {
                ui.label(t(world, "label-select-chapter"));
            });
            return;
        };
        let Some(chapter) = seq.chapters.get(idx) else {
            ui.label(t(world, "label-invalid-chapter"));
            return;
        };

        let chapter = chapter.clone();
        let icon = crate::widgets::chapter_icon(&chapter);
        let type_name = chapter_type_label(&chapter);

        ui.heading(format!("{icon} {type_name}"));
        ui.separator();

        let mut edited_chapter = chapter.clone();
        let changed = render_chapter_properties(ui, &mut edited_chapter, world);

        if changed {
            apply_chapter_edit(world, idx, &edited_chapter);
            world.resource_mut::<UndoStack>().push(ModifyChapterAction {
                index: idx,
                old_chapter: chapter,
                new_chapter: edited_chapter,
            });
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Requires World access");
    }
}

fn apply_chapter_edit(world: &mut World, idx: usize, edited: &Chapter) {
    let mut state = world.resource_mut::<EditorSequenceState>();
    if let Some(seq) = &mut state.current
        && let Some(ch) = seq.chapters.get_mut(idx)
    {
        *ch = edited.clone();
        seq.dirty = true;
    }
    state.save_timer = Some(0.5);
}
