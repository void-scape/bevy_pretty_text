use bevy::{
    core_pipeline::core_2d::{Transparent2d, CORE_2D_DEPTH_FORMAT},
    platform::collections::HashMap,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_phase::{AddRenderCommand, SetItemPipeline},
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
            DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, SpecializedRenderPipeline,
            SpecializedRenderPipelines, StencilFaceState, StencilState, TextureFormat,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        sync_world::MainEntityHashMap,
        view::ViewTarget,
        RenderApp,
    },
    sprite::{
        DrawMesh2d, Mesh2dPipeline, Mesh2dPipelineKey, RenderMesh2dInstance, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
    text::PositionedGlyph,
};

#[derive(Default, Resource)]
pub struct GlyphMeshCache(HashMap<GlyphHash, Handle<Mesh>>);

impl GlyphMeshCache {
    pub fn create_or_retrieve_mesh(
        &mut self,
        glyph: &PositionedGlyph,
        color: &LinearRgba,
        meshes: &mut Assets<Mesh>,
        atlases: &Assets<TextureAtlasLayout>,
    ) -> Handle<Mesh> {
        let hash = GlyphHash::from((glyph, color));
        self.0
            .entry(hash)
            .or_insert_with(|| meshes.add(create_glyph_mesh(glyph, color, atlases)))
            .clone()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphHash {
    atlas_id: AssetId<TextureAtlasLayout>,
    index: usize,
    color: [u8; 4],
}

impl From<(&PositionedGlyph, &LinearRgba)> for GlyphHash {
    fn from(value: (&PositionedGlyph, &LinearRgba)) -> Self {
        Self {
            atlas_id: value.0.atlas_info.texture_atlas.id(),
            index: value.0.atlas_info.location.glyph_index,
            color: value.1.to_u8_array(),
        }
    }
}

fn create_glyph_mesh(
    glyph: &PositionedGlyph,
    color: &LinearRgba,
    atlases: &Assets<TextureAtlasLayout>,
) -> Mesh {
    let [hw, hh] = [glyph.size.x / 2., glyph.size.y / 2.];
    let positions = vec![
        [hw, hh, 0.0],
        [-hw, hh, 0.0],
        [-hw, -hh, 0.0],
        [hw, -hh, 0.0],
    ];
    let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

    let atlas = atlases.get(&glyph.atlas_info.texture_atlas).unwrap();
    let uv_rect = atlas.textures[glyph.atlas_info.location.glyph_index].as_rect();
    let uv_rect = [
        uv_rect.min.x / atlas.size.x as f32,
        uv_rect.max.y / atlas.size.y as f32,
        uv_rect.max.x / atlas.size.x as f32,
        uv_rect.min.y / atlas.size.y as f32,
    ];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(indices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(
        MeshVertexAttribute::new("FontAtlasUvRect", 16, VertexFormat::Float32x4),
        VertexAttributeValues::Float32x4(vec![uv_rect; 4]),
    )
    .with_inserted_attribute(
        MeshVertexAttribute::new("TextColor", 17, VertexFormat::Float32x4),
        VertexAttributeValues::Float32x4(vec![color.to_f32_array(); 4]),
    )
}

#[derive(Clone, Resource)]
pub struct TextMesh2dPipeline {
    pub mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for TextMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

impl SpecializedRenderPipeline for TextMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            vec![
                VertexFormat::Float32x3,
                VertexFormat::Float32x4,
                VertexFormat::Float32x4,
            ],
        );
        vertex_layout.attributes.get_mut(1).unwrap().shader_location = 16;
        vertex_layout.attributes.get_mut(2).unwrap().shader_location = 17;

        let format = match key.contains(Mesh2dPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: TEXT_MESH2D_SHADER_HANDLE,
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: TEXT_MESH2D_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                self.mesh2d_pipeline.view_layout.clone(),
                self.mesh2d_pipeline.mesh_layout.clone(),
            ],
            push_constant_ranges: vec![],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("colored_mesh2d_pipeline".into()),
            zero_initialize_workgroup_memory: false,
        }
    }
}

type DrawColoredMesh2d = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    DrawMesh2d,
);

const TEXT_MESH2D_SHADER: &str = r"
#import bevy_sprite::mesh2d_functions

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(16) uv_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let model = mesh2d_functions::get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(vertex.position, 1.0));

    // out.color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    out.color = vertex.uv_rect;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // return in.color;
    return vec4(1., 1., 1., 1.);
}
";

pub struct TextMesh2dPlugin;

pub const TEXT_MESH2D_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(13828845428412094821);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderColoredMesh2dInstances(MainEntityHashMap<RenderMesh2dInstance>);

impl Plugin for TextMesh2dPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            &TEXT_MESH2D_SHADER_HANDLE,
            Shader::from_wgsl(TEXT_MESH2D_SHADER, file!()),
        );

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_command::<Transparent2d, DrawColoredMesh2d>()
            .init_resource::<SpecializedRenderPipelines<TextMesh2dPipeline>>()
            .init_resource::<RenderColoredMesh2dInstances>();
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<TextMesh2dPipeline>();
    }
}
