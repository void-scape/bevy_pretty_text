use bevy::prelude::*;

pub mod clear;
pub mod input;
pub mod scroll;
pub mod section;

pub struct TypeWriterPlugin;

impl Plugin for TypeWriterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<scroll::ScrollTimeout>()
            .add_event::<input::Input>()
            .add_systems(
                Update,
                (
                    input::read_input.before(TypeWriterSets::Update),
                    (
                        scroll::restart_changed_sections,
                        (
                            scroll::insert_scroll_timers,
                            scroll::insert_section_slices,
                            scroll::reveal_text_with_input,
                            scroll::propogate_char_sfx,
                            scroll::propogate_word_sfx,
                            scroll::propogate_rate_sfx,
                        ),
                        scroll::update_scroll_timer,
                        scroll::evaluate_scroll_timeout,
                        scroll::scroll_text,
                        scroll::handle_end,
                        section::update_section_slice,
                    )
                        .chain()
                        .in_set(TypeWriterSets::Update),
                    (clear::await_clear_section, clear::clear_section)
                        .in_set(TypeWriterSets::Clear),
                    section::update_section_slice_glyph_indices.after(TypeWriterSets::Update),
                    scroll::play_sfx
                        .before(scroll::scroll_text)
                        .after(scroll::evaluate_scroll_timeout),
                ),
            )
            .configure_sets(Update, TypeWriterSets::Clear.after(TypeWriterSets::Update));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeWriterSets {
    Update,
    Clear,
}
