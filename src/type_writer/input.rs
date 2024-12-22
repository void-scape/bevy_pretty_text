use super::scroll::Scroll;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Input {
    Interact,
}

#[derive(Component)]
pub struct InteractJustPressed;

pub fn read_input(
    mut commands: Commands,
    default_action_query: Query<Entity, (With<Scroll>, Without<ActionState<Input>>)>,
    //custom_action_query: Query<(Entity, &ActionState<Input>), With<Scroll>>,
    input: Res<ActionState<Input>>,
    //mut keyboard: EventReader<KeyboardInput>,
) {
    for entity in default_action_query.iter() {
        if input.just_pressed(&Input::Interact) {
            commands.entity(entity).insert(InteractJustPressed);
        }
    }

    // TODO: make leafwing work here
    //for (entity, input) in custom_action_query.iter() {
    //    if keyboard
    //        .read()
    //        .any(|i| i.state == ButtonState::Pressed && !i.repeat && i.key_code == KeyCode::Space)
    //    {
    //        commands.entity(entity).insert(InteractJustPressed);
    //    }
    //}
}
