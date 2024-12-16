//! Bevy Pretty Text is a simple plugin for rendering and encoding text effects.
//!
//! Use the [`ScrollBuilder`](crate::prelude::ScrollBuilder) to dynamically
//! construct scrolling text...
//! ```
//! # use bevy::prelude::*;
//! # use bevy_pretty_text::prelude::*;
//! #[derive(Component)]
//! struct MyScroll;
//!
//! fn system(mut commands: Commands) {
//!     ScrollBuilder::from_text("I am repeating!")
//!         .speed(1. / 5.) // 5 characters per second
//!         .mode(ScrollMode::Repeating)
//!         .spawn(&mut commands)
//!         .insert(MyScroll);
//! }
//! ```
//!
//! Or, simply add the [`Scroll`](crate::prelude::Scroll) component to any
//! entity that contains a [`TypeWriterSection`](crate::prelude::TypeWriterSection).
//! ```
//! # use bevy::prelude::*;
//! # use bevy_pretty_text::prelude::*;
//! (
//!     Scroll(1. / 2.),
//!     TypeWriterSection::from("Look at me!"),
//! );
//! ```
//!
//! If you want to directly controll what is displayed, then use a
//! [`SectionSlice`](crate::prelude::SectionSlice).
//! ```
//! # use bevy::prelude::*;
//! # use bevy_pretty_text::prelude::*;
//! (
//!     SectionSlice::All,
//!     TypeWriterSection::from("I am fully displayed!"),
//! );
//! ```
//!
//! A `TypeWriterSection` is just a Text2d hierarchy. This means that you interact
//! with it as a Text2d component, e.g.
//! ```
//! # use bevy::{prelude::*, text::*};
//! # use bevy_pretty_text::prelude::*;
//! (
//!     Scroll(1. / 2.),
//!     s!("`012`[wave]3`456789|blue`"),
//!     TextFont {
//!         font_size: 10.,
//!         font_smoothing: FontSmoothing::AntiAliased,
//!         ..Default::default()
//!     },
//!     TextBounds {
//!         width: Some(200.),
//!         height: Some(40.),
//!     },
//! );
//! ```

use self::materials::{ShakeMaterial, TextShaderPlugin};
use crate::effect::UpdateTextEffects;
use crate::materials::WaveMaterial;
use crate::render::material::TextMaterial2dPlugin;
use crate::render::mesh::{GlyphMeshCache, TextMesh2dPlugin};
use crate::type_writer::TypeWriterPlugin;
use bevy::prelude::*;
use bevy::text::Update2dText;

pub extern crate macros;
pub extern crate text;

pub mod effect;
pub mod materials;
pub mod render;
pub mod type_writer;

pub mod prelude {
    pub use crate::type_writer::{scroll::*, section::*};
    pub use crate::PrettyTextPlugin;
    pub use macros::s;
}

pub struct PrettyTextPlugin;

impl Plugin for PrettyTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TypeWriterPlugin,
            TextMesh2dPlugin,
            TextMaterial2dPlugin::<WaveMaterial>::default(),
            TextMaterial2dPlugin::<ShakeMaterial>::default(),
            TextShaderPlugin,
        ))
        .insert_resource(GlyphMeshCache::default())
        .add_systems(
            PostUpdate,
            (
                crate::effect::compute_info,
                crate::effect::extract_effect_glyphs,
            )
                .chain()
                .in_set(UpdateTextEffects)
                .after(Update2dText),
        );
    }
}
