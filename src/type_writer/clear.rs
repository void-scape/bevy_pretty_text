use super::section::TypeWriterSection;
use bevy::ecs::system::SystemId;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;

#[derive(Component)]
pub struct Clear;

#[derive(Component)]
pub struct DespawnOnClear(pub Entity);

#[derive(Component)]
pub struct DespawnOnParentClear;

pub fn clear_section(
    mut commands: Commands,
    section_query: Query<(Entity, Option<&Children>), (With<TypeWriterSection>, With<Clear>)>,
    despawn_query: Query<(Entity, &DespawnOnClear)>,
    child_despawn_query: Query<Entity, With<DespawnOnParentClear>>,
) {
    for (entity, children) in section_query.iter() {
        // If the section text is cleared, it would retain the old effect.
        //
        // Changing the TypeWriterSection, without removing it, forces the glyph system to rerun.
        // Clearing needs to have a stronger definition.
        commands
            .entity(entity)
            .insert(TypeWriterSection::default())
            .remove::<AwaitClear>();

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(entity) = child_despawn_query.get(*child) {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        for (despawn_entity, root_entity) in despawn_query.iter() {
            if entity == root_entity.0 {
                commands.entity(despawn_entity).despawn_recursive();
            }
        }
    }
}

#[derive(Default, Component)]
pub struct AwaitClear(Option<SystemId>);

impl AwaitClear {
    pub fn on_clear(id: SystemId) -> Self {
        Self(Some(id))
    }
}

pub fn await_clear_section(
    mut commands: Commands,
    // TODO: configurable input
    mut input: EventReader<KeyboardInput>,
    section_query: Query<(Entity, &AwaitClear), With<TypeWriterSection>>,
) {
    let received_input = input
        .read()
        .any(|i| i.state == ButtonState::Pressed && i.key_code == KeyCode::Space);

    if !received_input {
        return;
    }

    for (entity, await_clear) in section_query.iter() {
        commands.entity(entity).insert(Clear).remove::<AwaitClear>();
        if let Some(system) = await_clear.0 {
            commands.run_system(system);
        }
    }
}
