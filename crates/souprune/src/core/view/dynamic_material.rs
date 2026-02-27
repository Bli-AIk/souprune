//! # dynamic_material.rs
//!
//! Dynamic 2D Material system with runtime shader selection.
//! 支持运行时着色器选择的动态 2D 材质系统。
//!
//! This module implements a `SpecializedMeshPipeline`-based material system
//! that allows shaders to be specified at runtime via RON configuration.
//!
//! 本模块实现基于 `SpecializedMeshPipeline` 的材质系统，
//! 允许通过 RON 配置在运行时指定着色器。
//!
//! # Important: Material Batching Prevention
//! # 重要：防止材质合批
//!
//! Bevy's rendering pipeline batches entities that share the same mesh and
//! `Material2dBindGroupId` into a single draw call. For entities with different
//! material parameters (e.g., multiple HP bars with different HP values), this
//! causes all batched entities to render with the SAME material bind group
//! (typically the last entity's material).
//!
//! Bevy 的渲染管线会将共享相同 mesh 和 `Material2dBindGroupId` 的实体合批到一个 draw call。
//! 对于具有不同材质参数的实体（如多个血条有不同的 HP 值），这会导致所有合批的实体都使用
//! 相同的材质绑定组（通常是最后一个实体的材质）。
//!
//! **Solution**: In `queue_dynamic_material2d_meshes`, we must set each entity's
//! `material_bind_group_id` in `RenderMesh2dInstances` to prevent incorrect batching.
//!
//! **解决方案**：在 `queue_dynamic_material2d_meshes` 中，必须为每个实体在
//! `RenderMesh2dInstances` 中设置其 `material_bind_group_id` 以防止错误的合批。

use bevy::asset::{AssetId, Handle};
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::ecs::system::SystemParamItem;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::image::Image;
use bevy::math::{FloatOrd, Vec4};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::mesh::RenderMesh;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets};
use bevy::render::render_phase::{
    AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, RenderCommand, RenderCommandResult,
    SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
};
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor,
    BlendState, PipelineCache, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipeline,
    SpecializedMeshPipelineError, SpecializedMeshPipelines,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::sync_world::MainEntityHashMap;
use bevy::render::texture::GpuImage;
use bevy::render::view::ExtractedView;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::Shader;
use bevy::sprite_render::{
    DrawMesh2d, Material2dBindGroupId, Mesh2dPipeline, Mesh2dPipelineKey, RenderMesh2dInstances,
    SetMesh2dBindGroup, SetMesh2dViewBindGroup, init_mesh_2d_pipeline,
};
use bevy_shader::ShaderDefVal;
use std::collections::HashMap;
use std::hash::Hash;

// ============================================================================
// DynamicMaterial2d Asset
// ============================================================================

/// Uniform data structure for dynamic material parameters.
/// 动态材质参数的 uniform 数据结构。
#[derive(Clone, Default, ShaderType)]
pub struct DynamicMaterialUniform {
    /// Main parameters (vec4).
    /// 主参数 (vec4)。
    pub params: Vec4,

    /// Extra parameters (vec4).
    /// 额外参数 (vec4)。
    pub extra_params: Vec4,
}

/// Dynamic 2D Material - supports runtime shader specification.
/// 动态 2D 材质 - 支持运行时指定着色器。
///
/// Unlike standard `Material2d`, this asset stores the shader handle directly,
/// allowing different instances to use different shaders.
///
/// 与标准 `Material2d` 不同，此资产直接存储着色器句柄，
/// 允许不同实例使用不同的着色器。
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
#[uniform(0, DynamicMaterialUniform)]
pub struct DynamicMaterial2d {
    /// Runtime-loaded shader handle (not part of bind group).
    /// 运行时加载的着色器句柄（不属于绑定组）。
    pub shader: Handle<Shader>,

    /// Shader parameters (vec4).
    /// Components: (param0, param1, param2, param3)
    ///
    /// 着色器参数 (vec4)。
    /// 分量：(参数0, 参数1, 参数2, 参数3)
    pub params: Vec4,

    /// Extra shader parameters (vec4).
    /// For shaders that need more than 4 parameters.
    ///
    /// 额外着色器参数 (vec4)。
    /// 用于需要超过 4 个参数的着色器。
    pub extra_params: Vec4,

    /// Base texture.
    /// 基础纹理。
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl bevy::render::render_resource::AsBindGroupShaderType<DynamicMaterialUniform>
    for DynamicMaterial2d
{
    fn as_bind_group_shader_type(
        &self,
        _images: &RenderAssets<GpuImage>,
    ) -> DynamicMaterialUniform {
        DynamicMaterialUniform {
            params: self.params,
            extra_params: self.extra_params,
        }
    }
}

impl DynamicMaterial2d {
    /// Create a new dynamic material with default parameters.
    /// 使用默认参数创建新的动态材质。
    pub fn new(shader: Handle<Shader>, texture: Option<Handle<Image>>) -> Self {
        Self {
            shader,
            params: Vec4::new(1.0, 1.0, 0.0, 1.0),
            extra_params: Vec4::ZERO,
            texture,
        }
    }

    /// Create with custom params.
    /// 使用自定义参数创建。
    pub fn with_params(mut self, params: Vec4) -> Self {
        self.params = params;
        self
    }
}

// ============================================================================
// Material Component for Entity
// ============================================================================

/// Component to link an entity to a DynamicMaterial2d.
/// 将实体链接到 DynamicMaterial2d 的组件。
#[derive(Component, Clone, Debug, Deref, DerefMut, Default)]
pub struct MeshDynamicMaterial2d(pub Handle<DynamicMaterial2d>);

/// Debug component showing material AssetId for inspector.
/// 用于检查器的材质 AssetId 调试组件。
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MaterialAssetIdDebug {
    /// AssetId as string for inspector display.
    /// 作为字符串的 AssetId，用于检查器显示。
    pub asset_id: String,
}

// ============================================================================
// Pipeline Key
// ============================================================================

/// Pipeline key that includes shader asset ID for specialization.
/// 包含着色器资产 ID 的管道键，用于特化。
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DynamicMaterial2dKey {
    /// Base mesh pipeline key (MSAA, HDR, etc.).
    /// 基础 mesh 管道键（MSAA、HDR 等）。
    pub mesh_key: Mesh2dPipelineKey,

    /// Shader asset ID - different shaders produce different pipelines.
    /// 着色器资产 ID - 不同的着色器产生不同的管道。
    pub shader_id: AssetId<Shader>,
}

// ============================================================================
// Prepared Material (Render World)
// ============================================================================

/// Prepared DynamicMaterial2d for rendering.
/// 用于渲染的已准备 DynamicMaterial2d。
pub struct PreparedDynamicMaterial2d {
    /// The bind group for this material.
    /// 此材质的绑定组。
    pub bind_group: BindGroup,

    /// The shader handle for pipeline specialization.
    /// 用于管道特化的着色器句柄。
    pub shader: Handle<Shader>,

    /// Depth bias for sorting.
    /// 用于排序的深度偏移。
    pub depth_bias: f32,
}

impl RenderAsset for PreparedDynamicMaterial2d {
    type SourceAsset = DynamicMaterial2d;

    type Param = (
        SRes<RenderDevice>,
        SRes<DynamicMaterial2dPipeline>,
        SRes<PipelineCache>,
        <DynamicMaterial2d as AsBindGroup>::Param,
    );

    fn prepare_asset(
        material: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        (render_device, pipeline, pipeline_cache, material_param): &mut SystemParamItem<
            Self::Param,
        >,
        _previous: Option<&Self>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        // Use AsBindGroup to create the bind group
        // In Bevy 0.18, as_bind_group takes BindGroupLayoutDescriptor and PipelineCache
        match material.as_bind_group(
            &pipeline.material_layout_descriptor,
            render_device,
            pipeline_cache,
            material_param,
        ) {
            Ok(prepared) => Ok(PreparedDynamicMaterial2d {
                bind_group: prepared.bind_group,
                shader: material.shader.clone(),
                depth_bias: 0.0,
            }),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(material))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}

// ============================================================================
// Pipeline
// ============================================================================

/// Pipeline resource for DynamicMaterial2d.
/// DynamicMaterial2d 的管道资源。
#[derive(Resource)]
pub struct DynamicMaterial2dPipeline {
    /// Base mesh2d pipeline for shared layouts.
    /// 用于共享布局的基础 mesh2d 管道。
    pub mesh2d_pipeline: Mesh2dPipeline,

    /// Material bind group layout (generated by AsBindGroup).
    /// 材质绑定组布局（由 AsBindGroup 生成）。
    pub material_layout: BindGroupLayout,

    /// Material bind group layout descriptor for as_bind_group.
    /// 用于 as_bind_group 的材质绑定组布局描述符。
    pub material_layout_descriptor: BindGroupLayoutDescriptor,

    /// Cached shader handles for pipeline specialization.
    /// Key is shader asset ID, value is the handle.
    /// 用于管道特化的缓存着色器句柄。
    /// 键是着色器资产 ID，值是句柄。
    pub shader_cache: HashMap<AssetId<Shader>, Handle<Shader>>,
}

/// Initialize the DynamicMaterial2d pipeline.
/// 初始化 DynamicMaterial2d 管道。
pub fn init_dynamic_material2d_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mesh2d_pipeline: Res<Mesh2dPipeline>,
) {
    // Get both the layout and descriptor from AsBindGroup derive macro
    let material_layout = DynamicMaterial2d::bind_group_layout(&render_device);
    let material_layout_descriptor =
        DynamicMaterial2d::bind_group_layout_descriptor(&render_device);

    commands.insert_resource(DynamicMaterial2dPipeline {
        mesh2d_pipeline: mesh2d_pipeline.clone(),
        material_layout,
        material_layout_descriptor,
        shader_cache: HashMap::default(),
    });
}

impl SpecializedMeshPipeline for DynamicMaterial2dPipeline {
    type Key = DynamicMaterial2dKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        // Material bind group index (same as Bevy's Material2d)
        const MATERIAL_BIND_GROUP_INDEX: u32 = 2;

        // Get the base mesh2d descriptor
        let mut descriptor = self.mesh2d_pipeline.specialize(key.mesh_key, layout)?;

        // Add MATERIAL_BIND_GROUP shader def (required for #{MATERIAL_BIND_GROUP} placeholder)
        // 添加 MATERIAL_BIND_GROUP shader def（用于 #{MATERIAL_BIND_GROUP} 占位符）
        descriptor.vertex.shader_defs.push(ShaderDefVal::UInt(
            "MATERIAL_BIND_GROUP".into(),
            MATERIAL_BIND_GROUP_INDEX,
        ));
        if let Some(ref mut fragment) = descriptor.fragment {
            fragment.shader_defs.push(ShaderDefVal::UInt(
                "MATERIAL_BIND_GROUP".into(),
                MATERIAL_BIND_GROUP_INDEX,
            ));
        }

        // Get the shader handle from cache
        // Note: In specialize(), we can't load new shaders, so we use the cached handle.
        let shader_handle = self
            .shader_cache
            .get(&key.shader_id)
            .cloned()
            .unwrap_or_else(|| {
                warn!(
                    "Shader {:?} not found in cache, using default mesh2d shader",
                    key.shader_id
                );
                self.mesh2d_pipeline.shader.clone()
            });

        // Only override the fragment shader - keep mesh2d vertex shader
        // 只覆盖片段着色器 - 保留 mesh2d 顶点着色器
        if let Some(ref mut fragment) = descriptor.fragment {
            fragment.shader = shader_handle;
        }

        // Add material bind group layout descriptor
        descriptor
            .layout
            .push(self.material_layout_descriptor.clone());

        // Enable alpha blending for transparency
        if let Some(ref mut fragment) = descriptor.fragment
            && let Some(target) = fragment.targets.get_mut(0)
            && let Some(color_target) = target
        {
            color_target.blend = Some(BlendState::ALPHA_BLENDING);
        }

        descriptor.label = Some("dynamic_material2d_pipeline".into());
        Ok(descriptor)
    }
}

// ============================================================================
// Render World Resources
// ============================================================================

/// Extracted material instances in render world.
/// 渲染世界中提取的材质实例。
#[derive(Resource, Default, Deref, DerefMut)]
pub struct RenderDynamicMaterial2dInstances(MainEntityHashMap<AssetId<DynamicMaterial2d>>);

/// Extract material instances from main world to render world.
/// 从主世界提取材质实例到渲染世界。
pub fn extract_dynamic_material2d_instances(
    mut material_instances: ResMut<RenderDynamicMaterial2dInstances>,
    query: Extract<Query<(Entity, &ViewVisibility, &MeshDynamicMaterial2d)>>,
) {
    material_instances.clear();

    for (entity, visibility, material) in &query {
        if visibility.get() {
            material_instances.insert(entity.into(), material.0.id());
        }
    }
}

/// Cache shader handles from extracted materials.
/// 从提取的材质缓存着色器句柄。
pub fn cache_shader_handles(
    materials: Res<RenderAssets<PreparedDynamicMaterial2d>>,
    mut pipeline: ResMut<DynamicMaterial2dPipeline>,
) {
    for (_id, prepared) in materials.iter() {
        let shader_id = prepared.shader.id();
        pipeline
            .shader_cache
            .entry(shader_id)
            .or_insert_with(|| prepared.shader.clone());
    }
}

// ============================================================================
// Queue System
// ============================================================================

/// Queue dynamic material meshes for rendering.
/// 将动态材质网格入队进行渲染。
#[allow(clippy::too_many_arguments)]
pub fn queue_dynamic_material2d_meshes(
    pipeline: Res<DynamicMaterial2dPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedMeshPipelines<DynamicMaterial2dPipeline>>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_materials: Res<RenderAssets<PreparedDynamicMaterial2d>>,
    material_instances: Res<RenderDynamicMaterial2dInstances>,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    if material_instances.is_empty() {
        return;
    }

    let draw_function = draw_functions.read().id::<DrawDynamicMaterial2d>();

    for (view, msaa) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let view_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr)
            | Mesh2dPipelineKey::BLEND_ALPHA;

        for (visible_entity, material_asset_id) in material_instances.iter() {
            let Some(mesh_instance) = render_mesh_instances.get_mut(visible_entity) else {
                continue;
            };

            let Some(prepared_material) = render_materials.get(*material_asset_id) else {
                continue;
            };

            // CRITICAL: Set the material bind group ID to prevent batching across different materials.
            // Without this, Bevy batches all entities with the same mesh, causing them to share
            // the same material bind group (always using the last entity's material).
            // 关键：设置材质绑定组 ID 以防止不同材质的实体被合批。
            // 如果不设置，Bevy 会将所有使用相同 mesh 的实体合批，
            // 导致它们共享同一个材质绑定组（始终使用最后一个实体的材质）。
            mesh_instance.material_bind_group_id =
                Material2dBindGroupId(Some(prepared_material.bind_group.id()));

            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let mesh_key =
                view_key | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology());

            // Skip entities whose shader isn't cached yet to avoid permanently caching
            // a wrong pipeline via SpecializedMeshPipelines.
            if !pipeline
                .shader_cache
                .contains_key(&prepared_material.shader.id())
            {
                continue;
            }

            let key = DynamicMaterial2dKey {
                mesh_key,
                shader_id: prepared_material.shader.id(),
            };

            let pipeline_id =
                match pipelines.specialize(&pipeline_cache, &pipeline, key, &mesh.layout) {
                    Ok(id) => id,
                    Err(err) => {
                        trace!("Shader pipeline not ready, will retry: {}", err);
                        continue;
                    }
                };

            let mesh_z = mesh_instance.transforms.world_from_local.translation.z;

            transparent_phase.add(Transparent2d {
                entity: (Entity::PLACEHOLDER, *visible_entity),
                draw_function,
                pipeline: pipeline_id,
                sort_key: FloatOrd(mesh_z + prepared_material.depth_bias),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                extracted_index: usize::MAX,
                indexed: mesh.indexed(),
            });
        }
    }
}

// ============================================================================
// Draw Command
// ============================================================================

/// Draw command for DynamicMaterial2d.
/// DynamicMaterial2d 的绘制命令。
pub type DrawDynamicMaterial2d = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetDynamicMaterial2dBindGroup<2>,
    DrawMesh2d,
);

/// Render command to set the material bind group.
/// 设置材质绑定组的渲染命令。
pub struct SetDynamicMaterial2dBindGroup<const I: usize>;

impl<P: bevy::render::render_phase::PhaseItem, const I: usize> RenderCommand<P>
    for SetDynamicMaterial2dBindGroup<I>
{
    type Param = (
        SRes<RenderAssets<PreparedDynamicMaterial2d>>,
        SRes<RenderDynamicMaterial2dInstances>,
    );
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (materials, material_instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let materials = materials.into_inner();
        let material_instances = material_instances.into_inner();

        let Some(material_asset_id) = material_instances.get(&item.main_entity()) else {
            trace!(
                "[Render] Entity {:?} not found in material_instances",
                item.main_entity()
            );
            return RenderCommandResult::Skip;
        };

        let Some(material) = materials.get(*material_asset_id) else {
            trace!(
                "[Render] Material {:?} not found for entity {:?}",
                material_asset_id,
                item.main_entity()
            );
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &material.bind_group, &[]);
        RenderCommandResult::Success
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// Plugin for the DynamicMaterial2d system.
/// DynamicMaterial2d 系统的插件。
pub struct DynamicMaterial2dPlugin;

impl Plugin for DynamicMaterial2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<DynamicMaterial2d>()
            .register_type::<MaterialAssetIdDebug>()
            .add_plugins(RenderAssetPlugin::<PreparedDynamicMaterial2d>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<RenderDynamicMaterial2dInstances>()
            .init_resource::<SpecializedMeshPipelines<DynamicMaterial2dPipeline>>()
            .add_render_command::<Transparent2d, DrawDynamicMaterial2d>()
            .add_systems(
                RenderStartup,
                init_dynamic_material2d_pipeline.after(init_mesh_2d_pipeline),
            )
            .add_systems(ExtractSchedule, extract_dynamic_material2d_instances)
            .add_systems(
                Render,
                (
                    // Run after PrepareAssets so RenderAssetPlugin has processed shaders
                    cache_shader_handles.in_set(RenderSystems::PrepareResources),
                    queue_dynamic_material2d_meshes.in_set(RenderSystems::QueueMeshes),
                ),
            );
    }
}
