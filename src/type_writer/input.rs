use super::scroll::Scroll;
use bevy::prelude::*;

/// Signals interaction for any active scrolling textboxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Event)]
pub enum Input {
    Interact,
}

#[derive(Component)]
pub struct InteractJustPressed;

pub fn read_input(
    mut commands: Commands,
    mut reader: EventReader<Input>,
    default_action_query: Query<Entity, With<Scroll>>,
) {
    if !reader.is_empty() {
        reader.clear();
        for entity in default_action_query.iter() {
            commands.entity(entity).insert(InteractJustPressed);
        }
    }
}
