use super::section::*;
use bevy::prelude::*;
use std::time::Duration;

/// Scrolls through a [`TypeWriterSection`] with a specified `character per second` speed.
#[derive(Component)]
#[require(TypeWriterIndex, ScrollMode, ScrollTimer)]
pub struct Scroll(pub f32);

impl Scroll {
    pub fn from_section(section: TypeWriterSection) -> ScrollBuilder {
        ScrollBuilder::default().section(section)
    }
}

#[derive(Component)]
pub struct ScrollBuilder {
    speed: f32,
    mode: ScrollMode,
    section: TypeWriterSection,
}

impl ScrollBuilder {
    pub fn section(mut self, section: TypeWriterSection) -> Self {
        self.section = section;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn mode(mut self, mode: ScrollMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn spawn<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((Scroll(self.speed), self.mode, self.section))
    }
}

impl Default for ScrollBuilder {
    fn default() -> Self {
        Self {
            section: Default::default(),
            mode: Default::default(),
            speed: 1. / 20.,
        }
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
    scroll_query: Query<(Entity, &Scroll), (Added<Scroll>, With<TypeWriterSection>)>,
) {
    for (entity, scroll) in scroll_query.iter() {
        commands.entity(entity).insert(ScrollTimer::new(scroll.0));
    }
}

#[derive(Debug, Clone, Copy, Event)]
pub struct ScrollTimeout(Entity);

pub fn update_scroll_timer(
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ScrollTimer)>,
    mut writer: EventWriter<ScrollTimeout>,
) {
    for (entity, mut timer) in timers.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            writer.send(ScrollTimeout(entity));
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

pub fn scroll_text(
    mut reader: EventReader<ScrollTimeout>,
    mut text_query: Query<(&mut TypeWriterIndex, &TypeWriterSection, &ScrollMode)>,
) {
    for event in reader.read() {
        if let Ok((mut index, section, mode)) = text_query.get_mut(event.0) {
            match mode {
                ScrollMode::Once => {
                    if index.0 < section.len() {
                        index.0 += 1;
                    }
                }
                ScrollMode::Repeating => {
                    if index.0 < section.len() {
                        index.0 += 1;
                    } else {
                        index.0 = 0;
                    }
                }
            }
        }
    }
}
