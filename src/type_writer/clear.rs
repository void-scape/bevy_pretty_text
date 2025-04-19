use super::input::InteractJustPressed;
use super::section::TypeWriterSection;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

#[derive(Component)]
pub struct Clear;

#[derive(Component)]
pub struct DespawnOnClear(pub Entity);

#[derive(Component)]
pub struct DespawnOnParentClear;

pub fn clear_section(
    mut commands: Commands,
    section_query: Query<
        (Entity, Option<&Children>, Option<&OnClear>),
        (With<TypeWriterSection>, With<Clear>),
    >,
    despawn_query: Query<(Entity, &DespawnOnClear)>,
    child_despawn_query: Query<Entity, With<DespawnOnParentClear>>,
) {
    for (entity, children, on_clear) in section_query.iter() {
        // If the section text is cleared, it would retain the old effect.
        //
        // Changing the TypeWriterSection, without removing it, forces the glyph system to rerun.
        // Clearing needs to have a stronger definition.
        commands
            .entity(entity)
            .insert(TypeWriterSection::default())
            .remove::<(AwaitClear, Clear)>();

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

        if let Some(OnClear(system)) = on_clear {
            commands.run_system(*system);
        }
    }
}

#[derive(Component)]
pub struct OnClear(pub SystemId);

/// A typewriter with `AwaitClear` will await the [`Input`] event before clearing and continuing.
#[derive(Default, Component)]
pub struct AwaitClear;

pub fn await_clear_section(
    mut commands: Commands,
    section_query: Query<
        Entity,
        (
            With<TypeWriterSection>,
            With<InteractJustPressed>,
            With<AwaitClear>,
        ),
    >,
) {
    for entity in section_query.iter() {
        commands
            .entity(entity)
            .insert(Clear)
            .remove::<(AwaitClear, InteractJustPressed)>();
    }
}
