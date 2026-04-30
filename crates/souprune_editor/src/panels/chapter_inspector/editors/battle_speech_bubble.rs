//! Battle speech bubble chapter editor.
//!
//! 战斗对话气泡章节编辑器。
//!
//! Renders the editor controls for `Chapter::BattleSpeechBubble`, keeping the
//! top-level chapter editor below the repository file size limit.
//!
//! 渲染 `Chapter::BattleSpeechBubble` 的编辑控件，并让顶层章节编辑器保持在
//! 仓库文件大小限制之内。

use bevy::prelude::*;
use souprune_schema::battle::{BattleSpeechBubbleAdvance, BattleSpeechBubbleDef};

use crate::widgets;
use crate::widgets::property_editors::{
    edit_option_f32, edit_option_string, labeled_drag, labeled_text,
};

pub(super) fn edit_battle_speech_bubble(
    ui: &mut egui::Ui,
    bubble: &mut BattleSpeechBubbleDef,
    world: &World,
) -> bool {
    let mut changed = false;

    changed |= labeled_text(ui, "Channel", &mut bubble.channel);
    changed |=
        widgets::path_picker::edit_file_path(ui, "Mortar path", &mut bubble.mortar_path, world);
    changed |= labeled_text(ui, "Mortar node", &mut bubble.mortar_node);
    ui.label(format!("Frame: {:?}", bubble.frame));

    changed |= edit_advance_mode(ui, &mut bubble.advance);
    changed |= ui
        .checkbox(&mut bubble.hide_on_finish, "Hide on finish")
        .changed();
    changed |= edit_option_string(ui, "Voice", &mut bubble.voice);
    changed |= edit_option_f32(ui, "Typewriter speed", &mut bubble.typewriter_speed);

    changed
}

fn edit_advance_mode(ui: &mut egui::Ui, advance: &mut BattleSpeechBubbleAdvance) -> bool {
    let mut changed = false;
    let current_mode = match advance {
        BattleSpeechBubbleAdvance::Manual => 0,
        BattleSpeechBubbleAdvance::Timed { .. } => 1,
    };
    let mut selected_mode = current_mode;
    egui::ComboBox::from_label("Advance")
        .selected_text(match selected_mode {
            0 => "Manual",
            _ => "Timed",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut selected_mode, 0, "Manual");
            ui.selectable_value(&mut selected_mode, 1, "Timed");
        });
    if selected_mode != current_mode {
        *advance = match selected_mode {
            0 => BattleSpeechBubbleAdvance::Manual,
            _ => BattleSpeechBubbleAdvance::Timed { duration: 2.0 },
        };
        changed = true;
    }
    if let BattleSpeechBubbleAdvance::Timed { duration } = advance {
        changed |= labeled_drag(ui, "Duration", duration, 0.0..=60.0, 0.1);
    }

    changed
}
