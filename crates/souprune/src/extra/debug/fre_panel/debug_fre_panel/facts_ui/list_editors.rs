use bevy_inspector_egui::egui;

/// Shared editable string list UI. Returns `Some(new_list)` if changed.
pub(super) fn edit_string_list(ui: &mut egui::Ui, list: &[String]) -> Option<Vec<String>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            let response = ui.add(egui::TextEdit::singleline(item).desired_width(120.0));
            if response.changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(String::new());
        changed = true;
    }

    changed.then_some(new_list)
}

/// Shared editable int list UI. Returns `Some(new_list)` if changed.
pub(super) fn edit_int_list(ui: &mut egui::Ui, list: &[i64]) -> Option<Vec<i64>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            if ui.add(egui::DragValue::new(item)).changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(0);
        changed = true;
    }

    changed.then_some(new_list)
}

/// Shared editable float list UI. Returns `Some(new_list)` if changed.
pub(super) fn edit_float_list(ui: &mut egui::Ui, list: &[f64]) -> Option<Vec<f64>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            if ui.add(egui::DragValue::new(item).speed(0.1)).changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(0.0);
        changed = true;
    }

    changed.then_some(new_list)
}

/// Shared editable bool list UI. Returns `Some(new_list)` if changed.
pub(super) fn edit_bool_list(ui: &mut egui::Ui, list: &[bool]) -> Option<Vec<bool>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            if ui.checkbox(item, "").changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(false);
        changed = true;
    }

    changed.then_some(new_list)
}
