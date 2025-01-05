use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy_pretty_macro::text_shader;
use text::material::TextMaterial2d;

pub const WAVE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(140760193724908016942612779440926461879);

#[text_shader]
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Wave {
    #[uniform(2)]
    pub speed: f32,
}

impl TextMaterial2d for Wave {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        WAVE_SHADER_HANDLE.into()
    }
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
