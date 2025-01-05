use crate::impl_text_material2d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use text::material::TextMaterial2d;

#[derive(Component, Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Wave {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub speed: f32,
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            speed: 4.,
        }
    }
}

impl Wave {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Default::default()
        }
    }
}

pub const WAVE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(140760193724908016942612779440926461879);
impl_text_material2d!(Wave, WAVE_SHADER_HANDLE);
