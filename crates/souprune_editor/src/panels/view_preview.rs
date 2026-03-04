//! View 编辑器预览渲染系统。
//!
//! 复用游戏的 ViewBox/SDF/Text3d 渲染管线，通过 RenderLayers::layer(31) 隔离。

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use souprune::core::view::reconcile::{SpawnContext, ViewElementSpec, build_text_config};
use souprune::core::view::CameraAnchored;

use super::view_editor::ViewEditorState;

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
        }
    }
}

/// 标记预览实体。
#[derive(Component)]
pub struct ViewPreviewEntity;

/// 标记预览相机。
#[derive(Component)]
pub struct ViewPreviewCamera;

const PREVIEW_LAYER: usize = 31;

/// 初始化预览渲染目标和相机。
pub fn setup_view_preview(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<ViewPreviewState>,
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

    // 生成预览相机
    let camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -2,
                clear_color: ClearColorConfig::Custom(Color::srgba(0.12, 0.12, 0.16, 1.0)),
                ..default()
            },
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
    mortar_strings: Res<souprune::extra::mortar::MortarStringTable>,
    item_registry: Res<souprune::ItemRegistry>,
    fact_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
) {
    let Some(layout) = &editor_state.layout else {
        cleanup_preview(&mut commands, &mut preview_state);
        return;
    };

    // 简单脏检查：用指针地址 + dirty 作哈希
    let hash = layout as *const _ as u64;
    if hash == preview_state.last_layout_hash && !editor_state.dirty {
        return;
    }
    preview_state.last_layout_hash = hash;

    // 清理旧实体
    cleanup_preview(&mut commands, &mut preview_state);

    // 构建 SpawnContext — 使用模拟 camera_transform（预览相机在原点）
    let cam_transform = Transform::default();
    let player_data =
        souprune::core::view::ron_view::player_data::PlayerDataView::new(&fact_db);
    let ctx = SpawnContext::new(
        &asset_server,
        &mortar_strings,
        player_data,
        &item_registry,
        &cam_transform,
        editor_state
            .file_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("preview"),
    );

    // 遍历根节点，使用游戏管线生成预览实体
    for root in &layout.roots {
        spawn_preview_node(
            &mut commands,
            &mut preview_state,
            &ctx,
            root,
            None,
            0.0,
        );
    }
}

fn cleanup_preview(commands: &mut Commands, state: &mut ViewPreviewState) {
    for entity in state.preview_entities.drain(..) {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

fn spawn_preview_node(
    commands: &mut Commands,
    state: &mut ViewPreviewState,
    ctx: &SpawnContext,
    node: &souprune::core::view::layout::ViewNodeDef,
    parent: Option<Entity>,
    z_offset: f32,
) {
    let spec = ViewElementSpec {
        full_name: format!("preview:{}", node.name),
        local_name: node.name.clone(),
        namespace: "preview".to_string(),
        tags: node.tags.clone(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, z_offset)),
        visibility: Visibility::Inherited,
        visible_when_expr: node.visible_when.clone(),
        camera_anchored: false,
        camera_offset: Vec3::ZERO,
    };

    // ViewBox: 使用游戏管线的 spawn_viewbox_entity
    if let Some(vb) = &node.view_box {
        let texts: Vec<_> = node
            .texts
            .iter()
            .map(|td| build_text_config(td, ctx))
            .collect();
        let entity = souprune::core::view::reconcile::spawn_viewbox_entity(
            commands,
            parent,
            ctx,
            &spec,
            vb,
            texts,
            false, // is_top_level=false 避免 CameraAnchored
        );
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
        let entity = souprune::core::view::reconcile::spawn_sprite_entity(
            commands,
            parent,
            ctx,
            &spec,
            _sprite_def,
            None,
        );
        commands
            .entity(entity)
            .insert((RenderLayers::layer(PREVIEW_LAYER), ViewPreviewEntity));
        // 移除 CameraAnchored（spawn_sprite_entity 可能添加了它）
        commands.entity(entity).remove::<CameraAnchored>();
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

    // 容器中的文本：使用 spawn_text_entity 并移除 CameraAnchored
    for text_def in &node.texts {
        let entity = souprune::core::view::reconcile::spawn_text_entity(
            commands,
            Some(container),
            ctx,
            text_def,
            None,
        );
        commands
            .entity(entity)
            .insert((RenderLayers::layer(PREVIEW_LAYER), ViewPreviewEntity))
            .remove::<CameraAnchored>();
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

/// 在 UI 中渲染预览纹理，支持滚轮缩放和拖拽平移。
pub fn render_preview_ui(ui: &mut egui::Ui, state: &mut ViewPreviewState) {
    if let Some(tex_id) = state.egui_texture_id {
        let available = ui.available_size();
        let res = state.resolution;
        let aspect = res.x as f32 / res.y as f32;
        let base_size = if available.x / available.y > aspect {
            egui::vec2(available.y * aspect, available.y)
        } else {
            egui::vec2(available.x, available.x / aspect)
        };

        // 工具栏
        ui.horizontal(|ui| {
            ui.label(format!("Zoom: {:.0}%", state.zoom * 100.0));
            if ui.small_button("重置").clicked() {
                state.zoom = 1.0;
                state.pan_offset = Vec2::ZERO;
            }
        });

        let (rect, response) =
            ui.allocate_exact_size(base_size, egui::Sense::click_and_drag());

        // 滚轮缩放
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let factor = 1.0 + scroll * 0.002;
                state.zoom = (state.zoom * factor).clamp(0.1, 10.0);
            }
        }

        // 中键或右键拖拽平移
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            let delta = response.drag_delta();
            state.pan_offset.x += delta.x / state.zoom;
            state.pan_offset.y -= delta.y / state.zoom;
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
                "Preview (初始化中...)",
            );
        });
    }
}

/// 同步预览相机的缩放和平移。
pub fn sync_preview_camera(
    state: Res<ViewPreviewState>,
    mut cameras: Query<&mut Transform, With<ViewPreviewCamera>>,
) {
    if !state.is_changed() {
        return;
    }
    let Some(cam_entity) = state.camera_entity else {
        return;
    };
    if let Ok(mut transform) = cameras.get_mut(cam_entity) {
        transform.translation.x = -state.pan_offset.x;
        transform.translation.y = -state.pan_offset.y;
        let s = 1.0 / state.zoom;
        transform.scale = Vec3::new(s, s, 1.0);
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
        propagate_layers_recursive(
            &mut commands,
            root,
            &children_query,
            &without_layer,
            &layer,
        );
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
