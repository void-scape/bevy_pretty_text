use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    sprite::Anchor,
    text::{FontSmoothing, TextBounds},
};
use bevy_pretty_text::prelude::*;
use bevy_pretty_text::PrettyTextPlugin;

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

    Scroll::from_section(s!("`012`[wave]3`456789|green`"))
        .speed(1. / 5.)
        .mode(ScrollMode::Repeating)
        .spawn(&mut commands)
        .insert(Transform::from_scale(Vec3::splat(4.)).with_translation(Vec3::splat(100.)));

    commands.spawn((
        Scroll(1. / 2.),
        Anchor::TopLeft,
        s!("`012`[wave]3`456789|blue`"),
        //Text2d::new("Hello, World"),
        Transform::from_scale(Vec3::splat(4.)).with_translation(Vec3::new(-500., 300., 0.)),
        TextFont {
            font_size: 10.,
            font_smoothing: FontSmoothing::AntiAliased,
            ..Default::default()
        },
        TextBounds {
            width: Some(200.),
            height: Some(40.),
        },
    ));

    commands.spawn((
        SectionSlice::All,
        s!("<0.5> `He`[wave]`llo|red``, W`[shake]orl`d!`[shake]"),
        Transform::from_scale(Vec3::splat(4.)).with_translation(Vec3::splat(-100.)),
    ));
}
