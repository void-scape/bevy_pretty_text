use crate::TextMod;
use bevy::{
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    sprite::{AlphaMode2d, Mesh2dPipelineKey},
};
use std::hash::Hash;

pub trait InsertTextMaterial2d: CloneMod + Send + Sync + 'static {
    fn insert(&self, entity_commands: &mut EntityCommands);
}

#[derive(Component)]
pub struct CacheMaterial<M> {
    pub material: M,
}

impl<T: TextMaterial2d + Component + Clone> InsertTextMaterial2d for T {
    fn insert(&self, entity_commands: &mut EntityCommands) {
        entity_commands.insert(CacheMaterial {
            material: self.clone(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheType {
    Global,
    Local,
    Unique,
}

pub trait TextMaterial2d: AsBindGroup + Asset + Clone + Sized {
    /// Necessary in order to force the texture atlas to sync with the gpu.
    ///
    /// TODO: force the update without this method?
    fn set_texture(&mut self, texture: Handle<Image>);

    fn cache_type() -> CacheType {
        CacheType::Unique
    }

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

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    /// Customizes the default [`RenderPipelineDescriptor`].
    #[allow(unused_variables)]
    #[inline]
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: TextMaterial2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

pub struct TextMaterial2dKey<M: TextMaterial2d> {
    pub mesh_key: Mesh2dPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: TextMaterial2d> Eq for TextMaterial2dKey<M> where M::Data: PartialEq {}

impl<M: TextMaterial2d> PartialEq for TextMaterial2dKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.mesh_key == other.mesh_key && self.bind_group_data == other.bind_group_data
    }
}

impl<M: TextMaterial2d> Clone for TextMaterial2dKey<M>
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

impl<M: TextMaterial2d> Hash for TextMaterial2dKey<M>
where
    M::Data: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.mesh_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

pub trait CloneMod {
    fn clone_mod(&self) -> TextMod;
}

impl<T: InsertTextMaterial2d + Clone> CloneMod for T {
    fn clone_mod(&self) -> TextMod {
        TextMod::Shader(Box::new(self.clone()))
    }
}
