use crate::render::mesh::TextMesh2dPipeline;
use bevy::app::{App, Plugin};
use bevy::asset::{Asset, AssetApp, AssetId, AssetServer, Handle};
use bevy::core_pipeline::{
    core_2d::{AlphaMask2d, Opaque2d, Transparent2d},
    tonemapping::{DebandDither, Tonemapping},
};
use bevy::ecs::{
    prelude::*,
    system::{SystemParamItem, lifetimeless::SRes},
};
use bevy::image::Image;
use bevy::log::error;
use bevy::math::FloatOrd;
use bevy::prelude::{Deref, DerefMut, Mesh2d};
use bevy::reflect::{Reflect, prelude::ReflectDefault};
use bevy::render::render_resource::SpecializedRenderPipeline;
use bevy::render::sync_world::MainEntityHashMap;
use bevy::render::view::RenderVisibleEntities;
use bevy::render::{
    Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    mesh::{MeshVertexBufferLayoutRef, RenderMesh},
    render_asset::{
        PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets, prepare_assets,
    },
    render_phase::{
        AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
        RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
    },
    render_resource::{
        AsBindGroup, AsBindGroupError, BindGroup, BindGroupLayout, OwnedBindingResource,
        PipelineCache, RenderPipelineDescriptor, Shader, ShaderRef, SpecializedMeshPipeline,
        SpecializedMeshPipelineError, SpecializedMeshPipelines,
    },
    renderer::RenderDevice,
    view::{ExtractedView, Msaa, ViewVisibility},
};
use bevy::sprite::{
    AlphaMode2d, DrawMesh2d, Material2dBindGroupId, Mesh2dPipelineKey, RenderMesh2dInstances,
    SetMesh2dBindGroup, SetMesh2dViewBindGroup,
};
use core::{hash::Hash, marker::PhantomData};

pub trait TextMaterial2d: AsBindGroup + Asset + Clone + Sized {
    /// Necessary in order to force the texture atlas to sync with the gpu.
    ///
    /// TODO: force the update without this method?
    fn set_texture(&mut self, texture: Handle<Image>);

    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Add a bias to the view depth of the mesh which can be used to force a specific render order.
    #[inline]
    fn depth_bias(&self) -> f32 {
        0.0
    }

    /// Customizes the default [`RenderPipelineDescriptor`].
    #[allow(unused_variables)]
    #[inline]
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct TextMeshMaterial2d<M: TextMaterial2d>(pub Handle<M>);

impl<M: TextMaterial2d> Default for TextMeshMaterial2d<M> {
    fn default() -> Self {
        Self(Handle::default())
    }
}

impl<M: TextMaterial2d> From<TextMeshMaterial2d<M>> for AssetId<M> {
    fn from(material: TextMeshMaterial2d<M>) -> Self {
        material.id()
    }
}

impl<M: TextMaterial2d> From<&TextMeshMaterial2d<M>> for AssetId<M> {
    fn from(material: &TextMeshMaterial2d<M>) -> Self {
        material.id()
    }
}

pub struct TextMaterial2dPlugin<M: TextMaterial2d>(PhantomData<M>);

impl<M: TextMaterial2d> Default for TextMaterial2dPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: TextMaterial2d> Plugin for TextMaterial2dPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<M>()
            .register_type::<TextMeshMaterial2d<M>>()
            .add_plugins(RenderAssetPlugin::<PreparedTextMaterial2d<M>>::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque2d, DrawTextMaterial2d<M>>()
                .add_render_command::<AlphaMask2d, DrawTextMaterial2d<M>>()
                .add_render_command::<Transparent2d, DrawTextMaterial2d<M>>()
                .init_resource::<RenderTextMaterial2dInstances<M>>()
                .init_resource::<SpecializedMeshPipelines<TextMaterial2dPipeline<M>>>()
                .add_systems(ExtractSchedule, extract_mesh_materials_2d::<M>)
                .add_systems(
                    Render,
                    queue_material2d_meshes::<M>
                        .in_set(RenderSet::QueueMeshes)
                        .after(prepare_assets::<PreparedTextMaterial2d<M>>),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<TextMaterial2dPipeline<M>>();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct RenderTextMaterial2dInstances<M: TextMaterial2d>(MainEntityHashMap<AssetId<M>>);

impl<M: TextMaterial2d> Default for RenderTextMaterial2dInstances<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

fn extract_mesh_materials_2d<M: TextMaterial2d>(
    mut material_instances: ResMut<RenderTextMaterial2dInstances<M>>,
    query: Extract<Query<(Entity, &ViewVisibility, &TextMeshMaterial2d<M>), With<Mesh2d>>>,
) {
    material_instances.clear();

    for (entity, view_visibility, material) in &query {
        if view_visibility.get() {
            material_instances.insert(entity.into(), material.id());
        }
    }
}

#[derive(Resource)]
pub struct TextMaterial2dPipeline<M: TextMaterial2d> {
    pub mesh2d_pipeline: TextMesh2dPipeline,
    pub material2d_layout: BindGroupLayout,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

pub struct Material2dKey<M: TextMaterial2d> {
    pub mesh_key: Mesh2dPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: TextMaterial2d> Eq for Material2dKey<M> where M::Data: PartialEq {}

impl<M: TextMaterial2d> PartialEq for Material2dKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.mesh_key == other.mesh_key && self.bind_group_data == other.bind_group_data
    }
}

impl<M: TextMaterial2d> Clone for Material2dKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mesh_key: self.mesh_key,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: TextMaterial2d> Hash for Material2dKey<M>
where
    M::Data: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.mesh_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

impl<M: TextMaterial2d> Clone for TextMaterial2dPipeline<M> {
    fn clone(&self) -> Self {
        Self {
            mesh2d_pipeline: self.mesh2d_pipeline.clone(),
            material2d_layout: self.material2d_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: PhantomData,
        }
    }
}

impl<M: TextMaterial2d> SpecializedMeshPipeline for TextMaterial2dPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = Material2dKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh2d_pipeline.specialize(key.mesh_key);
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }

        descriptor.layout = vec![
            self.mesh2d_pipeline.mesh2d_pipeline.view_layout.clone(),
            self.mesh2d_pipeline.mesh2d_pipeline.mesh_layout.clone(),
            self.material2d_layout.clone(),
        ];

        M::specialize(&mut descriptor, layout, key)?;
        Ok(descriptor)
    }
}

impl<M: TextMaterial2d> FromWorld for TextMaterial2dPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();
        let material2d_layout = M::bind_group_layout(render_device);

        TextMaterial2dPipeline {
            mesh2d_pipeline: world.resource::<TextMesh2dPipeline>().clone(),
            material2d_layout,
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

pub(super) type DrawTextMaterial2d<M> = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetMaterial2dBindGroup<M, 2>,
    DrawMesh2d,
);

pub struct SetMaterial2dBindGroup<M: TextMaterial2d, const I: usize>(PhantomData<M>);
impl<P: PhaseItem, M: TextMaterial2d, const I: usize> RenderCommand<P>
    for SetMaterial2dBindGroup<M, I>
{
    type Param = (
        SRes<RenderAssets<PreparedTextMaterial2d<M>>>,
        SRes<RenderTextMaterial2dInstances<M>>,
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
        let Some(material_instance) = material_instances.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let Some(material2d) = materials.get(*material_instance) else {
            return RenderCommandResult::Skip;
        };
        // pass.set_vertex_buffer(1, material2d.vertex_buffer.slice(..));
        pass.set_bind_group(I, &material2d.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub const fn tonemapping_pipeline_key(tonemapping: Tonemapping) -> Mesh2dPipelineKey {
    match tonemapping {
        Tonemapping::None => Mesh2dPipelineKey::TONEMAP_METHOD_NONE,
        Tonemapping::Reinhard => Mesh2dPipelineKey::TONEMAP_METHOD_REINHARD,
        Tonemapping::ReinhardLuminance => Mesh2dPipelineKey::TONEMAP_METHOD_REINHARD_LUMINANCE,
        Tonemapping::AcesFitted => Mesh2dPipelineKey::TONEMAP_METHOD_ACES_FITTED,
        Tonemapping::AgX => Mesh2dPipelineKey::TONEMAP_METHOD_AGX,
        Tonemapping::SomewhatBoringDisplayTransform => {
            Mesh2dPipelineKey::TONEMAP_METHOD_SOMEWHAT_BORING_DISPLAY_TRANSFORM
        }
        Tonemapping::TonyMcMapface => Mesh2dPipelineKey::TONEMAP_METHOD_TONY_MC_MAPFACE,
        Tonemapping::BlenderFilmic => Mesh2dPipelineKey::TONEMAP_METHOD_BLENDER_FILMIC,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_material2d_meshes<M: TextMaterial2d>(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    material2d_pipeline: Res<TextMaterial2dPipeline<M>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<TextMaterial2dPipeline<M>>>,
    pipeline_cache: Res<PipelineCache>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_materials: Res<RenderAssets<PreparedTextMaterial2d<M>>>,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
    render_material_instances: Res<RenderTextMaterial2dInstances<M>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    views: Query<(
        &ExtractedView,
        &RenderVisibleEntities,
        &Msaa,
        Option<&Tonemapping>,
        Option<&DebandDither>,
    )>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    if render_material_instances.is_empty() {
        return;
    }

    for (view, visible_entities, msaa, tonemapping, dither) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let draw_transparent_2d = transparent_draw_functions
            .read()
            .id::<DrawTextMaterial2d<M>>();

        let mut view_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        if !view.hdr {
            if let Some(tonemapping) = tonemapping {
                view_key |= Mesh2dPipelineKey::TONEMAP_IN_SHADER;
                view_key |= tonemapping_pipeline_key(*tonemapping);
            }
            if let Some(DebandDither::Enabled) = dither {
                view_key |= Mesh2dPipelineKey::DEBAND_DITHER;
            }
        }
        for (render_entity, visible_entity) in visible_entities.iter::<With<Mesh2d>>() {
            let Some(material_asset_id) = render_material_instances.get(visible_entity) else {
                continue;
            };
            let Some(mesh_instance) = render_mesh_instances.get_mut(visible_entity) else {
                continue;
            };
            let Some(material_2d) = render_materials.get(*material_asset_id) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let mesh_key = view_key
                | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology())
                | material_2d.properties.mesh_pipeline_key_bits;

            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &material2d_pipeline,
                Material2dKey {
                    mesh_key,
                    bind_group_data: material_2d.key.clone(),
                },
                &mesh.layout,
            );

            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };

            mesh_instance.material_bind_group_id = material_2d.get_bind_group_id();
            let mesh_z = mesh_instance.transforms.world_from_local.translation.z;

            match material_2d.properties.alpha_mode {
                AlphaMode2d::Blend => {
                    transparent_phase.add(Transparent2d {
                        entity: (*render_entity, *visible_entity),
                        draw_function: draw_transparent_2d,
                        pipeline: pipeline_id,
                        sort_key: FloatOrd(mesh_z + material_2d.properties.depth_bias),
                        batch_range: 0..1,
                        extra_index: PhaseItemExtraIndex::None,
                        indexed: false,
                        extracted_index: 0,
                    });
                }
                _ => unreachable!(),
            }
        }
    }
}

pub struct Material2dProperties {
    pub alpha_mode: AlphaMode2d,
    pub depth_bias: f32,
    pub mesh_pipeline_key_bits: Mesh2dPipelineKey,
}

pub struct PreparedTextMaterial2d<T: TextMaterial2d> {
    #[allow(unused)]
    pub bindings: Vec<(u32, OwnedBindingResource)>,
    pub bind_group: BindGroup,
    pub key: T::Data,
    pub properties: Material2dProperties,
}

impl<T: TextMaterial2d> PreparedTextMaterial2d<T> {
    pub fn get_bind_group_id(&self) -> Material2dBindGroupId {
        Material2dBindGroupId(Some(self.bind_group.id()))
    }
}

impl<M: TextMaterial2d> RenderAsset for PreparedTextMaterial2d<M> {
    type SourceAsset = M;

    type Param = (
        SRes<RenderDevice>,
        SRes<TextMaterial2dPipeline<M>>,
        M::Param,
    );

    fn prepare_asset(
        material: Self::SourceAsset,
        _: AssetId<Self::SourceAsset>,
        (render_device, pipeline, material_param): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        match material.as_bind_group(&pipeline.material2d_layout, render_device, material_param) {
            Ok(prepared) => {
                let mut mesh_pipeline_key_bits = Mesh2dPipelineKey::empty();
                mesh_pipeline_key_bits.insert(Mesh2dPipelineKey::BLEND_ALPHA);
                Ok(PreparedTextMaterial2d {
                    bindings: prepared.bindings.0,
                    bind_group: prepared.bind_group,
                    key: prepared.data,
                    properties: Material2dProperties {
                        depth_bias: material.depth_bias(),
                        alpha_mode: AlphaMode2d::Blend,
                        mesh_pipeline_key_bits,
                    },
                })
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(material))
            }
            Err(other) => Err(PrepareAssetError::AsBindGroupError(other)),
        }
    }
}
