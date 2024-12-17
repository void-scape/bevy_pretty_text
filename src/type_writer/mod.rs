use crate::effect::UpdateTextEffects;
use bevy::prelude::*;

pub mod clear;
pub mod scroll;
pub mod section;

pub struct TypeWriterPlugin;

impl Plugin for TypeWriterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<scroll::ScrollTimeout>()
            .add_systems(
                Update,
                (
                    section::debug_sections,
                    scroll::insert_scroll_timers,
                    scroll::insert_section_slices,
                    section::update_section_slice,
                    (
                        scroll::evaluate_scroll_timeout,
                        scroll::update_scroll_timer,
                        scroll::scroll_text,
                        scroll::trigger_callback,
                    )
                        .chain(),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    (clear::clear_section, clear::await_clear_section),
                    section::update_section_slice_glyph_indices,
                )
                    .chain()
                    .before(UpdateTextEffects),
            );
    }
}
