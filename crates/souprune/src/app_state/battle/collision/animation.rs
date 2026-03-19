use super::geometry::{retire_battle_box, split_rect_box};
use super::*;
use std::collections::VecDeque;

fn lerp_boundary(start: &BattleBoxBoundary, end: &BattleBoxBoundary, t: f32) -> BattleBoxBoundary {
    BattleBoxBoundary {
        center: start.center.lerp(end.center, t),
        half_size: start.half_size.lerp(end.half_size, t),
    }
}

pub(super) fn finalize_merged_battle_box(
    commands: &mut Commands,
    source_entities: &[Entity],
    children_query: &Query<&Children>,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    player_query: &mut Query<&mut BoundToBattleBox, With<BehaviorParams>>,
    source_boxes: &(String, String),
    result_box: &str,
    merged_boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
) {
    for entity in source_entities {
        retire_battle_box(commands, *entity, children_query);
    }
    spawn_standalone_box(
        commands,
        meshes,
        sdf_materials,
        color_materials,
        result_box,
        merged_boundary,
        visual_style,
    );
    for mut bound in player_query.iter_mut() {
        if bound.0 == source_boxes.0 || bound.0 == source_boxes.1 {
            bound.0 = result_box.to_string();
        }
    }
}

/// Spawn a standalone battle box entity with its own SDF visual.
pub(super) fn spawn_standalone_box(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    id: &str,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
) {
    let view_box = visual_style.to_view_box(boundary.half_size.x * 2.0, boundary.half_size.y * 2.0);
    let entity = commands
        .spawn((
            BattleBox,
            BattleBoxId(id.to_string()),
            BattleBoxState::default(),
            visual_style.clone(),
            AlightMotionBattleBoxBounds {
                width: boundary.half_size.x * 2.0,
                height: boundary.half_size.y * 2.0,
                center_offset: Vec2::ZERO,
            },
            Transform::from_translation(boundary.center.extend(0.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new(format!("BattleBox:{id}")),
        ))
        .id();

    spawn_view_box_sdf_children(
        commands,
        entity,
        &view_box,
        meshes,
        sdf_materials,
        color_materials,
    );
}

/// Spawn a standalone battle box entity without SDF children.
/// Used for animation where visual is added separately.
pub(super) fn spawn_standalone_box_entity(
    commands: &mut Commands,
    id: &str,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
) -> Entity {
    commands
        .spawn((
            BattleBox,
            BattleBoxId(id.to_string()),
            BattleBoxState::default(),
            visual_style.clone(),
            AlightMotionBattleBoxBounds {
                width: boundary.half_size.x * 2.0,
                height: boundary.half_size.y * 2.0,
                center_offset: Vec2::ZERO,
            },
            Transform::from_translation(boundary.center.extend(0.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new(format!("BattleBox:{id}")),
        ))
        .id()
}

/// System to animate battle box split animations.
/// 战斗框分裂动画系统。
pub(super) fn animate_battle_box_split_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut BattleBoxSplitAnimation)>,
    mut box_query: Query<(&mut Transform, &mut AlightMotionBattleBoxBounds)>,
    child_query: Query<&Children>,
    mut shape_query: Query<(
        &mut crate::core::view::sdf_shape::ViewSdfShape,
        &MeshMaterial2d<SdfMaterial>,
        &mut Mesh2d,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
) {
    for (anim_entity, mut anim) in animations.iter_mut() {
        anim.progress += time.delta_secs() / anim.duration;
        let t = anim.progress.min(1.0);

        let eased = anim.easing.sample(t);
        let current_visible_gap = anim.target_visible_gap * eased;
        let current_gap = anim
            .visual_style
            .boundary_gap_for_visible_gap(current_visible_gap);
        let (current_a, current_b) = split_rect_box(
            &anim.original_boundary,
            &anim.split_axis,
            anim.split_position,
            current_gap,
            anim.gap_policy,
        );

        if anim.progress >= 1.0 {
            apply_boundary_to_box(
                anim.box_entity_a,
                &current_a,
                &anim.visual_style,
                &mut box_query,
                &child_query,
                &mut shape_query,
                &mut meshes,
                &mut sdf_materials,
            );
            apply_boundary_to_box(
                anim.box_entity_b,
                &current_b,
                &anim.visual_style,
                &mut box_query,
                &child_query,
                &mut shape_query,
                &mut meshes,
                &mut sdf_materials,
            );
            commands.entity(anim_entity).despawn();
            continue;
        }

        apply_boundary_to_box(
            anim.box_entity_a,
            &current_a,
            &anim.visual_style,
            &mut box_query,
            &child_query,
            &mut shape_query,
            &mut meshes,
            &mut sdf_materials,
        );
        apply_boundary_to_box(
            anim.box_entity_b,
            &current_b,
            &anim.visual_style,
            &mut box_query,
            &child_query,
            &mut shape_query,
            &mut meshes,
            &mut sdf_materials,
        );
    }
}

/// System to animate battle box merge animations.
/// 战斗框合并动画系统。
pub(super) fn animate_battle_box_merge_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut BattleBoxMergeAnimation)>,
    mut player_query: Query<&mut BoundToBattleBox, With<BehaviorParams>>,
    mut box_query: Query<(&mut Transform, &mut AlightMotionBattleBoxBounds)>,
    child_query: Query<&Children>,
    mut shape_query: Query<(
        &mut crate::core::view::sdf_shape::ViewSdfShape,
        &MeshMaterial2d<SdfMaterial>,
        &mut Mesh2d,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for (anim_entity, mut anim) in animations.iter_mut() {
        anim.progress += time.delta_secs() / anim.duration;
        let t = anim.progress.min(1.0);
        let eased = anim.easing.sample(t);

        let current_a = lerp_boundary(&anim.start_boundary_a, &anim.target_boundary_a, eased);
        let current_b = lerp_boundary(&anim.start_boundary_b, &anim.target_boundary_b, eased);

        apply_boundary_to_box(
            anim.box_entity_a,
            &current_a,
            &anim.visual_style,
            &mut box_query,
            &child_query,
            &mut shape_query,
            &mut meshes,
            &mut sdf_materials,
        );
        apply_boundary_to_box(
            anim.box_entity_b,
            &current_b,
            &anim.visual_style,
            &mut box_query,
            &child_query,
            &mut shape_query,
            &mut meshes,
            &mut sdf_materials,
        );

        if anim.progress < 1.0 {
            continue;
        }

        retire_battle_box(&mut commands, anim.box_entity_a, &child_query);
        retire_battle_box(&mut commands, anim.box_entity_b, &child_query);

        spawn_standalone_box(
            &mut commands,
            &mut meshes,
            &mut sdf_materials,
            &mut color_materials,
            &anim.result_box,
            &anim.merged_boundary,
            &anim.visual_style,
        );

        for mut bound in player_query.iter_mut() {
            if bound.0 == anim.source_boxes.0 || bound.0 == anim.source_boxes.1 {
                bound.0 = anim.result_box.clone();
            }
        }

        commands.entity(anim_entity).despawn();
    }
}

fn apply_boundary_to_box(
    box_entity: Entity,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
    box_query: &mut Query<(&mut Transform, &mut AlightMotionBattleBoxBounds)>,
    child_query: &Query<&Children>,
    shape_query: &mut Query<(
        &mut crate::core::view::sdf_shape::ViewSdfShape,
        &MeshMaterial2d<SdfMaterial>,
        &mut Mesh2d,
    )>,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
) {
    if let Ok((mut transform, mut bounds)) = box_query.get_mut(box_entity) {
        transform.translation = boundary.center.extend(0.0);
        bounds.width = boundary.half_size.x * 2.0;
        bounds.height = boundary.half_size.y * 2.0;
        update_sdf_visual(
            &box_entity,
            boundary,
            visual_style,
            child_query,
            shape_query,
            meshes,
            sdf_materials,
        );
    }
}

/// Update SDF visual for a battle box entity during animation.
fn update_sdf_visual(
    box_entity: &Entity,
    boundary: &BattleBoxBoundary,
    visual_style: &BattleBoxVisualStyle,
    child_query: &Query<&Children>,
    shape_query: &mut Query<(
        &mut crate::core::view::sdf_shape::ViewSdfShape,
        &MeshMaterial2d<SdfMaterial>,
        &mut Mesh2d,
    )>,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
) {
    let Ok(children) = child_query.get(*box_entity) else {
        return;
    };

    let mut queue: VecDeque<Entity> = VecDeque::from(children.to_vec());
    let mut sdf_entities = Vec::new();
    let expected_shapes = if visual_style.structure_file.is_some() {
        2
    } else {
        1
    };

    while let Some(entity) = queue.pop_front() {
        if shape_query.get(entity).is_ok() {
            sdf_entities.push(entity);
            if sdf_entities.len() >= expected_shapes {
                break;
            }
        }
        if let Ok(grandchildren) = child_query.get(entity) {
            queue.extend(grandchildren.to_vec());
        }
    }

    let box_half_width = boundary.half_size.x;
    let box_half_height = boundary.half_size.y;

    for (index, entity) in sdf_entities.into_iter().enumerate() {
        let (half_width, half_height) = if expected_shapes == 1 || index > 0 {
            (box_half_width, box_half_height)
        } else {
            (
                box_half_width + visual_style.border_width,
                box_half_height + visual_style.border_width,
            )
        };

        if let Ok((mut shape, material_handle, mut mesh_handle)) = shape_query.get_mut(entity) {
            shape.half_width = half_width;
            shape.half_height = half_height;
            if let Some(material) = sdf_materials.get_mut(&material_handle.0) {
                *material = shape.to_material();
            }
            mesh_handle.0 = meshes.add(shape.create_mesh());
        }
    }
}
