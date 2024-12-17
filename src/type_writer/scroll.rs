use super::section::*;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use std::time::Duration;
use text::TypeWriterCommand;

/// Scrolls through a [`TypeWriterSection`] with a specified `character per second` speed.
#[derive(Component)]
#[require(TypeWriterIndex, ScrollMode, ScrollTimer)]
pub struct Scroll(pub f32);

impl Scroll {
    pub const DEFAULT_SPEED: f32 = 1. / 20.;
}

impl Default for Scroll {
    fn default() -> Self {
        Self(Self::DEFAULT_SPEED)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Component)]
pub enum ScrollMode {
    #[default]
    Once,
    Repeating,
}

#[derive(Debug, Clone, Component)]
pub struct ScrollTimer(pub Timer);

impl Default for ScrollTimer {
    fn default() -> Self {
        Self::new(1. / 20.)
    }
}

impl ScrollTimer {
    pub fn new(characters_per_sec: f32) -> Self {
        Self(Timer::new(
            Duration::from_secs_f32(characters_per_sec),
            TimerMode::Repeating,
        ))
    }
}

pub fn insert_scroll_timers(
    mut commands: Commands,
    scroll_query: Query<(Entity, &Scroll), Changed<Scroll>>,
) {
    for (entity, scroll) in scroll_query.iter() {
        commands.entity(entity).insert(ScrollTimer::new(scroll.0));
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct ScrollTimeout(Entity);

#[derive(Component)]
pub struct Paused(Timer);

pub fn update_scroll_timer(
    mut commands: Commands,
    mut writer: EventWriter<ScrollTimeout>,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ScrollTimer), Without<Paused>>,
    mut paused_timers: Query<(Entity, &mut Paused)>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            writer.send(ScrollTimeout(entity));
        }
    }

    for (entity, mut paused) in paused_timers.iter_mut() {
        paused.0.tick(time.delta());
        if paused.0.finished() {
            commands
                .entity(entity)
                .remove::<Paused>()
                // Paused scrollers will always be ready to increment
                // immediately after unpausing
                .insert(IncrementIndex);
        }
    }
}

pub fn insert_section_slices(
    mut commands: Commands,
    index_query: Query<
        (Entity, &TypeWriterIndex),
        (Added<TypeWriterIndex>, With<TypeWriterSection>),
    >,
) {
    for (entity, index) in index_query.iter() {
        commands
            .entity(entity)
            .insert(SectionSlice::from_range(0..index.0));
    }
}

#[derive(Component)]
pub struct IncrementIndex;

pub fn evaluate_scroll_timeout(
    mut commands: Commands,
    mut reader: EventReader<ScrollTimeout>,
    text_query: Query<(Entity, &TypeWriterSection, &TypeWriterIndex)>,
) {
    for event in reader.read() {
        if let Ok((entity, section, index)) = text_query.get(event.0) {
            let mut entity = commands.entity(entity);
            let mut increment = true;

            for triggered in section.commands.iter().filter(|c| c.index == index.0) {
                match triggered.command {
                    TypeWriterCommand::Speed(s) => {
                        entity.insert(Scroll(s));
                    }
                    TypeWriterCommand::Pause(d) => {
                        entity.insert(Paused(Timer::from_seconds(d, TimerMode::Once)));
                        increment = false;
                    }
                    TypeWriterCommand::Delete(_) => {
                        unimplemented!()
                    }
                }
            }

            if increment {
                entity.insert(IncrementIndex);
            }
        }
    }
}

#[derive(Component)]
pub struct ScrollJustFinished;

pub fn scroll_text(
    mut commands: Commands,
    mut reader: EventReader<ScrollTimeout>,
    mut text_query: Query<
        (
            Entity,
            &mut TypeWriterIndex,
            &TypeWriterSection,
            &ScrollMode,
        ),
        With<IncrementIndex>,
    >,
) {
    for event in reader.read() {
        if let Ok((entity, mut index, section, mode)) = text_query.get_mut(event.0) {
            let mut entity = commands.entity(entity);

            let len = section.len();
            if index.0 < len {
                index.0 += 1;

                if index.0 == len {
                    entity.insert(ScrollJustFinished);
                }
            } else {
                if *mode == ScrollMode::Repeating {
                    index.0 = 0;
                }
            }

            entity.remove::<IncrementIndex>();
        }
    }
}

#[derive(Component)]
pub struct OnScrollEnd(pub SystemId);

pub fn trigger_callback(
    mut commands: Commands,
    finished_query: Query<(Entity, &OnScrollEnd), With<ScrollJustFinished>>,
) {
    for (entity, on_end) in finished_query.iter() {
        commands.run_system(on_end.0);
        commands.entity(entity).remove::<ScrollJustFinished>();
    }
}
