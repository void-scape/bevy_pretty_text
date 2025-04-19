use super::clear::AwaitClear;
use super::input::InteractJustPressed;
use super::section::*;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use std::{sync::Arc, time::Duration};
use text::TypeWriterCommand;

/// Scrolls through a [`TypeWriterSection`] with a specified `character per second` speed.
#[derive(Component)]
#[require(TypeWriterIndex, ScrollMode, ScrollTimer, Scrolling)]
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

#[derive(Default, Component)]
pub struct Scrolling;

#[derive(Component)]
pub struct Paused(Timer);

#[derive(Debug, Clone, Copy, Event)]
pub struct ScrollTimeout(Entity);

pub fn update_scroll_timer(
    mut commands: Commands,
    mut writer: EventWriter<ScrollTimeout>,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut ScrollTimer), (Without<Paused>, With<Scrolling>)>,
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
        (Entity, &TypeWriterIndex, &TypeWriterSection),
        (Added<TypeWriterIndex>, With<TypeWriterSection>),
    >,
) {
    for (entity, index, section) in index_query.iter() {
        commands
            .entity(entity)
            .insert(SectionSlice::from_range(0..index.0));

        if section.text.value.is_empty() {
            commands.entity(entity).insert(ScrollJustFinished);
        }
    }
}

#[derive(Component)]
pub struct IncrementIndex;

pub fn evaluate_scroll_timeout(
    mut commands: Commands,
    mut reader: EventReader<ScrollTimeout>,
    text_query: Query<(Entity, &TypeWriterSection, &TypeWriterIndex, &Scroll)>,
) {
    for event in reader.read() {
        if let Ok((entity, section, index, scroll)) = text_query.get(event.0) {
            let mut entity = commands.entity(entity);
            let mut increment = true;

            for triggered in section.commands.iter().filter(|c| c.index == index.0) {
                match triggered.command {
                    TypeWriterCommand::Speed(s) => {
                        entity.insert(ScrollTimer(Timer::from_seconds(
                            scroll.0 / s,
                            TimerMode::Repeating,
                        )));
                    }
                    TypeWriterCommand::Pause(d) => {
                        entity.insert(Paused(Timer::from_seconds(d, TimerMode::Once)));
                        increment = false;
                    }
                    TypeWriterCommand::Delete(_) => {
                        unimplemented!()
                    }
                    TypeWriterCommand::AwaitClear => {
                        entity.insert(AwaitClear);
                    }
                }
            }

            if increment {
                entity.insert(IncrementIndex);
            }
        }
    }
}

pub fn reveal_text_with_input(
    mut commands: Commands,
    mut text_query: Query<
        (
            Entity,
            &mut TypeWriterIndex,
            &TypeWriterSection,
            &ScrollMode,
        ),
        (With<Scrolling>, With<InteractJustPressed>),
    >,
) {
    for (entity, mut index, section, mode) in text_query.iter_mut() {
        index.0 = section.len();

        let mut entity = commands.entity(entity);
        entity
            .insert(ScrollJustFinished)
            .remove::<(IncrementIndex, Paused, InteractJustPressed)>();

        if *mode == ScrollMode::Once {
            entity.remove::<Scrolling>();
        }
    }
}

#[derive(Component)]
pub struct ScrollJustFinished;

pub fn scroll_text(
    mut commands: Commands,
    mut text_query: Query<
        (
            Entity,
            &mut TypeWriterIndex,
            &TypeWriterSection,
            &ScrollMode,
        ),
        (With<IncrementIndex>, With<Scrolling>),
    >,
) {
    for (entity, mut index, section, mode) in text_query.iter_mut() {
        let mut entity = commands.entity(entity);

        let len = section.len();
        if index.0 < len {
            index.0 += 1;

            if index.0 == len {
                entity.insert(ScrollJustFinished).remove::<Scrolling>();
            }
        } else {
            if *mode == ScrollMode::Repeating {
                index.0 = 0;
            }
        }

        entity.remove::<IncrementIndex>();
    }
}

pub trait ScrollSfx: Clone + Component {}

impl ScrollSfx for SfxChar {}
impl ScrollSfx for SfxWord {}
impl ScrollSfx for SfxRate {}

#[derive(Clone)]
struct SoundBundle(Arc<dyn Fn(&mut Commands) + Send + Sync + 'static>);

impl SoundBundle {
    fn new(bundle: impl Bundle + Clone) -> Self {
        Self(Arc::new(move |commands| {
            commands.spawn(bundle.clone());
        }))
    }
}

impl Default for SoundBundle {
    fn default() -> Self {
        Self(Arc::new(|_| ()))
    }
}

#[derive(Clone, Component, Default)]
pub struct SfxChar(SoundBundle);

impl SfxChar {
    pub fn new(bundle: impl Bundle + Clone) -> Self {
        Self(SoundBundle::new(bundle))
    }
}

#[derive(Clone, Component, Default)]
pub struct SfxWord(SoundBundle);

impl SfxWord {
    pub fn new(bundle: impl Bundle + Clone) -> Self {
        Self(SoundBundle::new(bundle))
    }
}

#[derive(Clone, Component)]
#[require(SfxRateAccumulator)]
pub struct SfxRate {
    pub rate: f32,
    bundle: SoundBundle,
}

impl SfxRate {
    pub fn new(rate: f32, bundle: impl Bundle + Clone) -> Self {
        Self {
            rate,
            bundle: SoundBundle::new(bundle),
        }
    }
}

impl Default for SfxRate {
    fn default() -> Self {
        Self {
            rate: Scroll::DEFAULT_SPEED,
            bundle: SoundBundle::default(),
        }
    }
}

#[derive(Debug, Default, Component)]
pub struct SfxRateAccumulator(f32);

pub fn propogate_char_sfx(
    mut commands: Commands,
    textbox_query: Query<(&Children, &SfxChar), Or<(Changed<SfxChar>, Changed<Children>)>>,
    text_query: Query<Entity, With<Scroll>>,
) {
    for (children, sfx) in textbox_query.iter() {
        for child in children.iter() {
            if let Ok(entity) = text_query.get(*child) {
                commands.entity(entity).insert(sfx.clone());
            }
        }
    }
}

pub fn propogate_word_sfx(
    mut commands: Commands,
    textbox_query: Query<(&Children, &SfxWord), Or<(Changed<SfxWord>, Changed<Children>)>>,
    text_query: Query<Entity, With<Scroll>>,
) {
    for (children, sfx) in textbox_query.iter() {
        for child in children.iter() {
            if let Ok(entity) = text_query.get(*child) {
                commands.entity(entity).insert(sfx.clone());
            }
        }
    }
}

pub fn propogate_rate_sfx(
    mut commands: Commands,
    textbox_query: Query<(&Children, &SfxRate), Or<(Changed<SfxRate>, Changed<Children>)>>,
    text_query: Query<Entity, With<Scroll>>,
) {
    for (children, sfx) in textbox_query.iter() {
        for child in children.iter() {
            if let Ok(entity) = text_query.get(*child) {
                commands.entity(entity).insert(sfx.clone());
            }
        }
    }
}

pub fn play_sfx(
    mut commands: Commands,
    text_query: Query<
        (Entity, &TypeWriterSection, &TypeWriterIndex),
        (With<IncrementIndex>, Without<Paused>),
    >,
    char_query: Query<&SfxChar>,
    word_query: Query<&SfxWord>,
    mut rate_query: Query<(&SfxRate, &mut SfxRateAccumulator), (With<Scrolling>, Without<Paused>)>,
    time: Res<Time>,
) {
    for (entity, section, index) in text_query.iter() {
        if let Ok(sfx) = char_query.get(entity) {
            let bytes = section.text.value.as_bytes();
            if bytes.get(index.0).is_some_and(|c| *c != b' ') {
                (sfx.0 .0)(&mut commands);
            }
        }

        if let Ok(sfx) = word_query.get(entity) {
            let bytes = section.text.value.as_bytes();
            if (index.0 == 0 && bytes.get(index.0).is_some_and(|c| *c != b' '))
                || (bytes
                    .get(index.0.saturating_sub(1))
                    .is_some_and(|c| *c == b' ')
                    && bytes.get(index.0).is_some_and(|c| *c != b' '))
            {
                (sfx.0 .0)(&mut commands);
            }
        }
    }

    for (sfx, mut accum) in rate_query.iter_mut() {
        accum.0 += time.delta_secs();
        if accum.0 >= sfx.rate {
            accum.0 -= sfx.rate;

            (sfx.bundle.0)(&mut commands);
        }
    }
}

#[derive(Component)]
pub struct OnScrollEnd(pub SystemId);

pub fn handle_end(
    mut commands: Commands,
    finished_query: Query<
        (Entity, Option<&OnScrollEnd>, &TypeWriterSection, &Scroll),
        With<ScrollJustFinished>,
    >,
) {
    for (entity, on_end, section, scroll) in finished_query.iter() {
        if let Some(OnScrollEnd(on_end)) = on_end {
            commands.run_system(*on_end);
        }

        let mut entity = commands.entity(entity);
        entity.remove::<ScrollJustFinished>();
        if let Some(command) = section.end {
            match command {
                TypeWriterCommand::Speed(s) => {
                    entity.insert(ScrollTimer(Timer::from_seconds(
                        scroll.0 / s,
                        TimerMode::Repeating,
                    )));
                }
                TypeWriterCommand::Pause(d) => {
                    entity.insert(Paused(Timer::from_seconds(d, TimerMode::Once)));
                }
                TypeWriterCommand::Delete(_) => {
                    unimplemented!()
                }
                TypeWriterCommand::AwaitClear => {
                    entity.insert(AwaitClear);
                }
            }
        }
    }
}

pub fn restart_changed_sections(
    mut commands: Commands,
    text_query: Query<Entity, (With<Scroll>, Changed<TypeWriterSection>)>,
) {
    for entity in text_query.iter() {
        commands
            .entity(entity)
            .remove::<(ScrollJustFinished, InteractJustPressed, Paused)>()
            .insert(Scrolling);
    }
}
