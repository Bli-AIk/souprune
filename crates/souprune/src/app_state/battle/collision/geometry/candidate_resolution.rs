use super::*;

/// Resolve the `BattleBoxBoundary` from a battle box entity.
/// Returns `None` if the box is inactive or collision is disabled.
pub(in crate::app_state::battle::collision) fn resolve_boundary(
    transform: &GlobalTransform,
    ui_box: Option<&ViewBox>,
    am_bounds: Option<&AlightMotionBattleBoxBounds>,
    state: &BattleBoxState,
) -> Option<BattleBoxBoundary> {
    if !state.active || !state.collision_enabled {
        return None;
    }
    if let Some(vb) = ui_box {
        Some(BattleBoxBoundary::from_ui_box(
            vb.width(),
            vb.height(),
            transform.translation().truncate(),
        ))
    } else if let Some(am) = am_bounds {
        let center = transform.translation().truncate() + am.center_offset;
        Some(BattleBoxBoundary::from_ui_box(am.width, am.height, center))
    } else {
        None
    }
}

fn resolve_visual_style(
    ui_box: Option<&ViewBox>,
    style: Option<&BattleBoxVisualStyle>,
) -> BattleBoxVisualStyle {
    style
        .cloned()
        .or_else(|| ui_box.map(BattleBoxVisualStyle::from_view_box))
        .unwrap_or_default()
}

fn collect_battle_box_candidates(
    target_id: &str,
    ui_boxes: &UiBattleBoxReadQuery,
    am_boxes: &AmBattleBoxReadQuery,
) -> Vec<BattleBoxCandidate> {
    let mut candidates = Vec::new();

    for (entity, tf, vb, box_id, state, style) in ui_boxes.iter() {
        if box_id.0 != target_id {
            continue;
        }

        candidates.push(BattleBoxCandidate {
            entity,
            id: box_id.0.clone(),
            kind: BattleBoxSourceKind::Ui,
            active: state.active,
            collision_enabled: state.collision_enabled,
            boundary: resolve_boundary(tf, Some(vb), None, state),
            visual_style: resolve_visual_style(Some(vb), style),
        });
    }

    for (entity, tf, am, box_id, state, style) in am_boxes.iter() {
        if box_id.0 != target_id {
            continue;
        }

        candidates.push(BattleBoxCandidate {
            entity,
            id: box_id.0.clone(),
            kind: BattleBoxSourceKind::Am,
            active: state.active,
            collision_enabled: state.collision_enabled,
            boundary: resolve_boundary(tf, None, Some(am), state),
            visual_style: resolve_visual_style(None, style),
        });
    }

    candidates
}

fn describe_battle_box_candidates(candidates: &[BattleBoxCandidate]) -> String {
    candidates
        .iter()
        .map(BattleBoxCandidate::summary)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(in crate::app_state::battle::collision) fn resolve_live_battle_box(
    op_name: &str,
    target_id: &str,
    ui_boxes: &UiBattleBoxReadQuery,
    am_boxes: &AmBattleBoxReadQuery,
) -> Option<BattleBoxCandidate> {
    let candidates = collect_battle_box_candidates(target_id, ui_boxes, am_boxes);
    if candidates.is_empty() {
        warn!("{op_name}: source box '{target_id}' not found");
        return None;
    }

    let live = candidates
        .iter()
        .filter(|candidate| candidate.is_live())
        .cloned()
        .collect::<Vec<_>>();

    match live.len() {
        1 => live.into_iter().next(),
        0 => {
            warn!(
                "{op_name}: source box '{target_id}' exists but has no usable live match: {}",
                describe_battle_box_candidates(&candidates)
            );
            None
        }
        count => {
            warn!(
                "{op_name}: source box '{target_id}' is ambiguous; found {count} live matches: {}",
                describe_battle_box_candidates(&live)
            );
            None
        }
    }
}

fn hide_entity_and_descendants(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    commands.entity(entity).insert(Visibility::Hidden);

    let Ok(children) = children_query.get(entity) else {
        return;
    };

    for child in children.iter() {
        hide_entity_and_descendants(commands, child, children_query);
    }
}

pub(in crate::app_state::battle::collision) fn retire_battle_box(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    hide_entity_and_descendants(commands, entity, children_query);
    commands
        .entity(entity)
        .remove::<(BattleBox, BattleBoxId, BattleBoxState, BattleBoxVisualStyle)>();
}

pub(in crate::app_state::battle::collision) fn retire_existing_battle_boxes_with_id(
    commands: &mut Commands,
    op_name: &str,
    target_id: &str,
    keep_entities: &[Entity],
    children_query: &Query<&Children>,
    ui_boxes: &UiBattleBoxReadQuery,
    am_boxes: &AmBattleBoxReadQuery,
) {
    for candidate in collect_battle_box_candidates(target_id, ui_boxes, am_boxes) {
        if keep_entities.contains(&candidate.entity) {
            continue;
        }

        warn!(
            "{op_name}: retiring pre-existing box for result id '{target_id}': {}",
            candidate.summary()
        );
        retire_battle_box(commands, candidate.entity, children_query);
    }
}
