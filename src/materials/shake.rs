use crate::impl_text_material2d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use text::material::TextMaterial2d;

#[derive(Component, Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Shake {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub intensity: f32,
}

impl Default for Shake {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            intensity: 0.5,
        }
    }
}

impl Shake {
    pub fn new(intensity: f32) -> Self {
        Self {
            intensity,
            ..Default::default()
        }
    }
}

pub const SHAKE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(79568255301670276214221003054688204202);
impl_text_material2d!(Shake, SHAKE_SHADER_HANDLE);
