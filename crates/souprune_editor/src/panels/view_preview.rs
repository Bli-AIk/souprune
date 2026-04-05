//! View 编辑器预览渲染系统。
//!
//! 复用游戏的 ViewBox/SDF/Text3d 渲染管线，通过 RenderLayers::layer(31) 隔离。

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use souprune::editor_api::{app, game_action, mortar, view};
use souprune_schema::view::{InitialFactValue, ViewLayoutAsset as SchemaViewLayoutAsset};
use view::{
    ActiveView, PlayerDataView, RepeatContext, SpawnContext, ViewElementSpec,
    ViewNodeDef as RuntimeViewNodeDef, ViewRoot, build_text_config, load_fre_into_view_root,
    spawn_sprite_entity, spawn_text_entity, spawn_viewbox_entity,
};

use super::view_editor::ViewEditorState;

mod play_control;

pub use play_control::{
    ViewPreviewKeyMap, preview_input_to_fre_system, preview_play_control_system,
};

/// 预览系统状态资源。
#[derive(Resource)]
pub struct ViewPreviewState {
    pub render_target: Handle<Image>,
    pub egui_texture_id: Option<egui::TextureId>,
    pub resolution: UVec2,
    pub camera_entity: Option<Entity>,
    /// 当前预览中的实体列表
    pub preview_entities: Vec<Entity>,
    /// 上次预览重建时的 layout 快照哈希（脏检查）
    pub last_layout_hash: u64,
    /// 预览相机缩放级别
    pub zoom: f32,
    /// 预览相机平移偏移
    pub pan_offset: Vec2,
    /// Resolution scale factor (cached from ResolutionScale for UI calculations)
    pub resolution_scale: f32,
    /// Preview FRE interaction active (Play mode)
    pub playing: bool,
    /// Previous value of `playing` for transition detection
    pub was_playing: bool,
    /// Whether the preview image is hovered (enables keyboard input forwarding)
    pub hovered: bool,
    /// Rule IDs registered during Play (cleaned up on Stop)
    pub registered_rule_ids: Vec<String>,
}

impl Default for ViewPreviewState {
    fn default() -> Self {
        Self {
            render_target: Handle::default(),
            egui_texture_id: None,
            resolution: UVec2::new(640, 480),
            camera_entity: None,
            preview_entities: Vec::new(),
            last_layout_hash: 0,
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            resolution_scale: 1.0,
            playing: false,
            was_playing: false,
            hovered: false,
            registered_rule_ids: Vec::new(),
        }
    }
}

/// 标记预览实体。
#[derive(Component)]
pub struct ViewPreviewEntity;

/// 标记预览相机。
#[derive(Component)]
pub struct ViewPreviewCamera;

/// Pre-translated labels for the preview toolbar (avoids borrow conflicts with World).
pub struct PreviewLabels {
    pub stop: String,
    pub play: String,
    pub zoom: String,
    pub reset: String,
    pub input_active: String,
}

const PREVIEW_LAYER: usize = 31;

/// 初始化预览渲染目标和相机。
pub fn setup_view_preview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<ViewPreviewState>,
    resolution_scale: Res<app::ResolutionScale>,
) {
    // 创建渲染目标纹理
    let mut image = Image::new_target_texture(
        state.resolution.x,
        state.resolution.y,
        TextureFormat::Bgra8UnormSrgb,
        Some(TextureFormat::Bgra8UnormSrgb),
    );
    image.sampler = ImageSampler::nearest();
    state.render_target = images.add(image);

    // 生成预览相机（使用与游戏相机一致的 OrthographicProjection scale）
    let camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -2,
                clear_color: ClearColorConfig::Custom(Color::srgba(0.12, 0.12, 0.16, 1.0)),
                ..default()
            },
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0 / resolution_scale.0 as f32,
                ..OrthographicProjection::default_2d()
            }),
            RenderTarget::from(state.render_target.clone()),
            RenderLayers::layer(PREVIEW_LAYER),
            ViewPreviewCamera,
        ))
        .id();
    state.camera_entity = Some(camera);
}

/// 注册 egui 纹理（懒初始化）。
pub fn sync_preview_texture(
    mut state: ResMut<ViewPreviewState>,
    mut contexts: bevy_egui::EguiContexts,
) {
    if state.egui_texture_id.is_none() && state.render_target != Handle::default() {
        let texture_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
            state.render_target.clone(),
        ));
        state.egui_texture_id = Some(texture_id);
    }
}

/// 当 ViewEditorState 的 layout 变化时，重建预览实体。
///
/// 复用游戏的 ViewBox/SDF 渲染管线，通过 SpawnContext + spawn_viewbox_entity 实现。
pub fn rebuild_preview_entities(
    mut commands: Commands,
    editor_state: Res<ViewEditorState>,
    mut preview_state: ResMut<ViewPreviewState>,
    asset_server: Res<AssetServer>,
    mortar_strings: Res<mortar::MortarStringTable>,
    fact_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
) {
    let Some(layout) = &editor_state.layout else {
        cleanup_preview(&mut commands, &mut preview_state);
        return;
    };
    let runtime_layout = match view::runtime_view_layout_from_schema(layout) {
        Ok(layout) => layout,
        Err(err) => {
            warn!("[ViewPreview] runtime layout conversion failed: {err}");
            cleanup_preview(&mut commands, &mut preview_state);
            return;
        }
    };

    // Dirty check using generation counter
    let generation = editor_state.generation;
    if generation == preview_state.last_layout_hash && !editor_state.dirty {
        return;
    }
    preview_state.last_layout_hash = generation;

    // Stop playing when preview rebuilds (FRE state becomes stale)
    if preview_state.playing {
        preview_state.playing = false;
    }

    // Cleanup old entities
    cleanup_preview(&mut commands, &mut preview_state);

    // Spawn a ViewRoot container for FRE Play mode
    let layout_path = editor_state
        .file_path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("preview")
        .to_string();
    let view_root_entity = commands
        .spawn((
            ViewRoot::new(layout_path),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Inherited,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            RenderLayers::layer(PREVIEW_LAYER),
            ViewPreviewEntity,
            Name::new("preview_view_root"),
        ))
        .id();
    preview_state.preview_entities.push(view_root_entity);

    // Build SpawnContext
    let cam_transform = Transform::default();
    let player_data = PlayerDataView::new(&fact_db);
    let ctx = SpawnContext::new(
        &asset_server,
        &mortar_strings,
        player_data,
        &cam_transform,
        editor_state
            .file_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("preview"),
    );

    // Spawn preview nodes parented to the ViewRoot entity
    for root in &runtime_layout.roots {
        spawn_preview_node(
            &mut commands,
            &mut preview_state,
            &ctx,
            root,
            Some(view_root_entity),
            0.0,
        );
    }
}

fn cleanup_preview(commands: &mut Commands, state: &mut ViewPreviewState) {
    // Only despawn root-level entities; Bevy cascades despawn to children.
    if let Some(root) = state.preview_entities.first()
        && let Ok(mut ec) = commands.get_entity(*root)
    {
        ec.despawn();
    }
    state.preview_entities.clear();
}

fn spawn_preview_node(
    commands: &mut Commands,
    state: &mut ViewPreviewState,
    ctx: &SpawnContext,
    node: &RuntimeViewNodeDef,
    parent: Option<Entity>,
    z_offset: f32,
) {
    spawn_preview_node_inner(commands, state, ctx, node, parent, z_offset, None);
}

fn spawn_preview_node_inner(
    commands: &mut Commands,
    state: &mut ViewPreviewState,
    ctx: &SpawnContext,
    node: &RuntimeViewNodeDef,
    parent: Option<Entity>,
    z_offset: f32,
    repeat_ctx: Option<&RepeatContext>,
) {
    // Handle repeat expansion: spawn N instances with RepeatContext
    if let Some(repeat_spec) = &node.repeat {
        let count = ctx
            .player_data
            .get_array_length(&format!("${}", repeat_spec.source))
            .unwrap_or(0);
        let limit = repeat_spec.limit.unwrap_or(usize::MAX);
        let count = count.min(limit);
        for i in 0..count {
            let rctx = RepeatContext::new(i);
            spawn_preview_node_inner(
                commands,
                state,
                ctx,
                node,
                parent,
                z_offset + 0.01 * i as f32,
                Some(&rctx),
            );
        }
        return;
    }

    let spec = ViewElementSpec {
        full_name: if let Some(rctx) = repeat_ctx {
            format!("preview:{}_{}", node.name, rctx.get_index())
        } else {
            format!("preview:{}", node.name)
        },
        local_name: node.name.clone(),
        namespace: "preview".to_string(),
        tags: node.tags.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, z_offset)),
        visibility: Visibility::Inherited,
        visible_when_expr: if let Some(rctx) = repeat_ctx {
            node.visible_when.as_ref().map(|expr| {
                expr.replace("@i", &rctx.get_index().to_string())
                    .replace("@index", &rctx.get_index().to_string())
            })
        } else {
            node.visible_when.clone()
        },
    };

    // ViewBox: 使用游戏管线的 spawn_viewbox_entity
    if let Some(vb) = &node.view_box {
        let texts: Vec<_> = node
            .texts
            .iter()
            .map(|td| build_text_config(td, ctx))
            .collect();
        let entity = spawn_viewbox_entity(commands, parent, ctx, &spec, vb, texts);
        commands
            .entity(entity)
            .insert((RenderLayers::layer(PREVIEW_LAYER), ViewPreviewEntity));
        state.preview_entities.push(entity);

        // 递归子节点，以 ViewBox 实体作为父级
        for (i, child) in node.children.iter().enumerate() {
            spawn_preview_node(
                commands,
                state,
                ctx,
                child,
                Some(entity),
                z_offset + 0.1 * (i + 1) as f32,
            );
        }
        return;
    }

    // Sprite: 使用游戏管线的 spawn_sprite_entity
    if let Some(_sprite_def) = &node.sprite {
        let entity = spawn_sprite_entity(commands, parent, ctx, &spec, _sprite_def, repeat_ctx);
        commands
            .entity(entity)
            .insert((RenderLayers::layer(PREVIEW_LAYER), ViewPreviewEntity));
        state.preview_entities.push(entity);

        for (i, child) in node.children.iter().enumerate() {
            spawn_preview_node(
                commands,
                state,
                ctx,
                child,
                Some(entity),
                z_offset + 0.1 * (i + 1) as f32,
            );
        }
        return;
    }

    // 纯容器节点：创建一个空的父实体
    let container = commands
        .spawn((
            Transform::from_translation(Vec3::new(0.0, 0.0, z_offset)),
            GlobalTransform::default(),
            spec.visibility,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            RenderLayers::layer(PREVIEW_LAYER),
            ViewPreviewEntity,
            Name::new(spec.local_name.clone()),
        ))
        .id();
    if let Some(p) = parent {
        commands.entity(p).add_child(container);
    }
    state.preview_entities.push(container);

    // 容器中的文本：使用 spawn_text_entity
    for text_def in &node.texts {
        let entity = spawn_text_entity(commands, Some(container), ctx, text_def, None);
        commands
            .entity(entity)
            .insert((RenderLayers::layer(PREVIEW_LAYER), ViewPreviewEntity));
        state.preview_entities.push(entity);
    }

    for (i, child) in node.children.iter().enumerate() {
        spawn_preview_node(
            commands,
            state,
            ctx,
            child,
            Some(container),
            z_offset + 0.1 * (i + 1) as f32,
        );
    }
}

fn render_preview_toolbar(ui: &mut egui::Ui, state: &mut ViewPreviewState, labels: &PreviewLabels) {
    if state.playing {
        if ui.small_button(&labels.stop).clicked() {
            state.playing = false;
        }
    } else if ui.small_button(&labels.play).clicked() {
        state.playing = true;
    }
    ui.separator();
    ui.label(&labels.zoom);
    if ui.small_button(&labels.reset).clicked() {
        state.zoom = 1.0;
        state.pan_offset = Vec2::ZERO;
    }
    if state.playing && state.hovered {
        ui.separator();
        ui.colored_label(egui::Color32::from_rgb(100, 255, 100), &labels.input_active);
    }
}

/// 在 UI 中渲染预览纹理，支持滚轮缩放和拖拽平移。
pub fn render_preview_ui(ui: &mut egui::Ui, state: &mut ViewPreviewState, labels: &PreviewLabels) {
    if let Some(tex_id) = state.egui_texture_id {
        let available = ui.available_size();
        let res = state.resolution;
        let aspect = res.x as f32 / res.y as f32;
        let base_size = if available.x / available.y > aspect {
            egui::vec2(available.y * aspect, available.y)
        } else {
            egui::vec2(available.x, available.x / aspect)
        };

        // Toolbar
        ui.horizontal(|ui| {
            render_preview_toolbar(ui, state, labels);
        });

        let (rect, response) = ui.allocate_exact_size(base_size, egui::Sense::click_and_drag());

        // Update hovered state from egui response (like GameViewFocus)
        state.hovered = response.hovered();

        // Scroll zoom
        if state.hovered {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let factor = 1.0 + scroll * 0.001;
                state.zoom = (state.zoom * factor).clamp(0.1, 10.0);
            }
        }

        // Middle or right button drag to pan
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = response.drag_delta();
            let scale = state.zoom * state.resolution_scale;
            state.pan_offset.x += delta.x / scale;
            state.pan_offset.y -= delta.y / scale;
        }

        // Hover highlight border (like Game View)
        if state.playing && state.hovered {
            let painter = ui.painter();
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255)),
                egui::StrokeKind::Outside,
            );
        }

        ui.painter().image(
            tex_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 40));
        ui.centered_and_justified(|ui| {
            ui.colored_label(
                egui::Color32::from_rgb(100, 100, 120),
                "Preview (initializing...)",
            );
        });
    }
}

/// 同步预览相机的缩放和平移。
pub fn sync_preview_camera(
    mut state: ResMut<ViewPreviewState>,
    resolution_scale: Res<app::ResolutionScale>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<ViewPreviewCamera>>,
) {
    // Cache resolution_scale for UI pan calculations
    let rs = resolution_scale.0 as f32;
    if (state.resolution_scale - rs).abs() > f32::EPSILON {
        state.resolution_scale = rs;
    }

    if !state.is_changed() {
        return;
    }
    let Some(cam_entity) = state.camera_entity else {
        return;
    };
    if let Ok((mut transform, mut projection)) = cameras.get_mut(cam_entity) {
        transform.translation.x = -state.pan_offset.x;
        transform.translation.y = -state.pan_offset.y;
        // 通过 OrthographicProjection::scale 实现缩放，保持与游戏相机一致的基准
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 1.0 / (resolution_scale.0 as f32 * state.zoom);
        }
    }
}

/// 将 RenderLayers::layer(PREVIEW_LAYER) 传播到 ViewPreviewEntity 的所有后代。
///
/// SDF 渲染系统和文本系统会自动创建子实体（Mesh2d、Text3d 等），
/// 这些子实体没有 RenderLayers，需要递归传播以确保在预览相机中可见。
pub fn propagate_preview_render_layers(
    mut commands: Commands,
    preview_entities: Query<Entity, With<ViewPreviewEntity>>,
    children_query: Query<&Children>,
    without_layer: Query<Entity, (Without<RenderLayers>, Without<ViewPreviewCamera>)>,
) {
    let layer = RenderLayers::layer(PREVIEW_LAYER);
    for root in preview_entities.iter() {
        propagate_layers_recursive(&mut commands, root, &children_query, &without_layer, &layer);
    }
}

fn propagate_layers_recursive(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
    without_layer: &Query<Entity, (Without<RenderLayers>, Without<ViewPreviewCamera>)>,
    layer: &RenderLayers,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            if without_layer.get(child).is_ok() {
                commands.entity(child).insert(layer.clone());
            }
            propagate_layers_recursive(commands, child, children_query, without_layer, layer);
        }
    }
}
