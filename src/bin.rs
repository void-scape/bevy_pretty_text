use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_pretty_text::{
    type_writer::*,
    PrettyTextPlugin,
};
use std::borrow::Cow;
use text::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PrettyTextPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, close_on_escape)
        .run();
}

fn close_on_escape(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for event in reader.read() {
        if event.state == ButtonState::Pressed && event.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let val = "0123456789";
    commands.spawn((
        //SectionSlice::All,
        Scroll(1. / 2.),
        TypeWriterSection {
            text: text::Text {
                value: Cow::Borrowed(val),
                modifiers: &[
                    IndexedTextMod {
                        start: 4,
                        end: 11,
                        text_mod: TextMod::Color(LinearRgba::GREEN),
                    },
                    IndexedTextMod {
                        start: 0,
                        end: 3,
                        text_mod: TextMod::Wave,
                    },
                ],
            },
            commands: &[],
        },
        Transform::from_scale(Vec3::splat(4.)).with_translation(Vec3::splat(100.)),
    ));

    let val = "Hello, World";
    commands.spawn((
        SectionSlice::All,
        TypeWriterSection {
            text: text::Text {
                value: Cow::Borrowed(val),
                modifiers: &[
                    IndexedTextMod {
                        start: 0,
                        end: 2,
                        text_mod: TextMod::Wave,
                    },
                    IndexedTextMod {
                        start: 5,
                        end: 8,
                        text_mod: TextMod::Wave,
                    },
                    IndexedTextMod {
                        start: 10,
                        end: 13,
                        text_mod: TextMod::Shake(0.1),
                    },
                    IndexedTextMod {
                        start: 0,
                        end: 3,
                        text_mod: TextMod::Color(LinearRgba::RED),
                    },
                ],
            },
            commands: &[], // &[IndexedCommand {
                           //     index: 13,
                           //     command: TypeWriterCommand::AwaitClear,
                           // }],
        },
        Transform::from_scale(Vec3::splat(4.)).with_translation(Vec3::splat(-100.)),
    ));
}
