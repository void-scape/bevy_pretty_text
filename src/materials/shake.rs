use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy_pretty_macro::text_shader;
use text::material::TextMaterial2d;

pub const SHAKE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(79568255301670276214221003054688204202);

#[text_shader]
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Shake {
    #[uniform(2)]
    pub intensity: f32,
}

impl TextMaterial2d for Shake {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SHAKE_SHADER_HANDLE.into()
    }
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
