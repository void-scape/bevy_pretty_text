use crate::effect::UpdateTextEffects;
use bevy::prelude::*;

pub mod scroll;
pub mod section;

pub struct TypeWriterPlugin;

impl Plugin for TypeWriterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<scroll::ScrollTimeout>()
            .add_systems(
                Update,
                (
                    section::debug_sections,
                    section::update_section_slice,
                    scroll::update_scroll_timer,
                    scroll::scroll_text,
                    scroll::insert_scroll_timers,
                    scroll::insert_section_slices,
                ),
            )
            .add_systems(
                PostUpdate,
                section::update_section_slice_glyph_indices.before(UpdateTextEffects),
            );
    }
}

//#[derive(Debug, Clone)]
//pub struct TypeWriterToken(pub TypeWriterSection);
//
//impl From<&'static str> for TypeWriterToken {
//    fn from(value: &'static str) -> Self {
//        Self(TypeWriterSection::from(value))
//    }
//}
//
//impl IntoFragment<TypeWriterToken, BoxContext> for &'static str {
//    fn into_fragment(self, context: &BoxContext, commands: &mut Commands) -> FragmentId {
//        <_ as IntoFragment<TypeWriterToken, _>>::into_fragment(
//            bevy_sequence::fragment::DataLeaf::new(TypeWriterToken::from(self)),
//            context,
//            commands,
//        )
//    }
//}

// // #[derive(Component, Debug, Clone)]
// // pub struct TypeWriterState {
// //     chars_per_sec: f32,
// //     state: State,
// //     effect_mapping: Vec<Option<bevy_bits::TextEffect>>,
// //     fragment_id: Option<FragmentId>,
// //     reveal_accum: f32,
// //     delete_accum: f32,
// //     section_accumulator: Vec<String>,
// // }
// //
// // impl Default for TypeWriterState {
// //     fn default() -> Self {
// //         Self {
// //             chars_per_sec: 20.0,
// //             state: State::Ready,
// //             effect_mapping: Vec::new(),
// //             fragment_id: None,
// //             reveal_accum: 0.0,
// //             delete_accum: 0.0,
// //             section_accumulator: Vec::new(),
// //         }
// //     }
// // }
//
// // impl TypeWriterState {
// //     pub fn new(chars_per_sec: f32) -> Self {
// //         Self {
// //             chars_per_sec,
// //             ..Default::default()
// //         }
// //     }
// //
// //     pub fn push_section(
// //         &mut self,
// //         section: bevy_bits::tokens::TextSection,
// //         id: Option<FragmentId>,
// //     ) {
// //         debug_assert!(!matches!(self.state, State::Sequence { .. }));
// //
// //         self.state = State::Section {
// //             section: TypeWriterSectionBuffer::new(section),
// //             timer: Timer::new(
// //                 Duration::from_secs_f32(1.0 / self.chars_per_sec),
// //                 TimerMode::Repeating,
// //             ),
// //         };
// //         self.fragment_id = id;
// //     }
// //
// //     pub fn push_cmd(&mut self, command: TextCommand, id: Option<FragmentId>) {
// //         debug_assert!(!matches!(self.state, State::Sequence { .. }));
// //
// //         self.state = State::Command(command);
// //         self.fragment_id = id;
// //     }
// //
// //     pub fn push_seq(&mut self, sequence: Cow<'static, [DialogueBoxToken]>, id: Option<FragmentId>) {
// //         debug_assert!(sequence.len() > 0);
// //
// //         let mut type_writer = TypeWriterState::new(self.chars_per_sec);
// //         match sequence[0].clone() {
// //             DialogueBoxToken::Section(sec) => {
// //                 type_writer.push_section(sec, Some(FragmentId::random()))
// //             }
// //             DialogueBoxToken::Command(cmd) => type_writer.push_cmd(cmd, Some(FragmentId::random())),
// //             DialogueBoxToken::Sequence(seq) => {
// //                 type_writer.push_seq(seq, Some(FragmentId::random()))
// //             }
// //         }
// //
// //         self.state = State::Sequence {
// //             sequence,
// //             type_writer: Box::new(type_writer),
// //             index: 1,
// //             force_update: false,
// //         };
// //         self.fragment_id = id;
// //     }
// //
// //     pub fn tick(
// //         &mut self,
// //         time: &Time,
// //         received_input: bool,
// //         text: &mut Text,
// //         box_font: &Font,
// //         commands: &mut Commands,
// //         reveal: Option<&super::audio::RevealedTextSfx>,
// //         delete: Option<&super::audio::DeletedTextSfx>,
// //         force_update: bool,
// //     ) -> Option<FragmentEndEvent> {
// //         let mut end_event = None;
// //         let new_state = match &mut self.state {
// //             State::Ready => None,
// //             State::Section { section, timer } => {
// //                 timer.tick(time.delta());
// //
// //                 if let Some(reveal) = reveal {
// //                     if matches!(reveal.settings.trigger, super::audio::Trigger::Rate(_)) {
// //                         self.reveal_accum += time.delta_seconds();
// //                     }
// //                 }
// //
// //                 if let Some(occurance) = timer.finished().then(|| section.advance()) {
// //                     Self::update_text(
// //                         text,
// //                         &mut self.effect_mapping,
// //                         &mut self.section_accumulator,
// //                         box_font,
// //                         occurance,
// //                     );
// //
// //                     if let Some(reveal) = reveal {
// //                         if !force_update {
// //                             Self::try_play_reveal(
// //                                 &self
// //                                     .section_accumulator
// //                                     .iter()
// //                                     .flat_map(|a| a.chars())
// //                                     .collect::<String>(),
// //                                 &mut self.reveal_accum,
// //                                 reveal,
// //                                 commands,
// //                             );
// //                         }
// //                     }
// //                 }
// //
// //                 section.finished().then(|| {
// //                     end_event = self.fragment_id;
// //                     State::Ready
// //                 })
// //             }
// //             State::Delete { amount, timer } => {
// //                 timer.tick(time.delta());
// //
// //                 if let Some(delete) = delete {
// //                     if matches!(delete.settings.trigger, super::audio::Trigger::Rate(_)) {
// //                         self.delete_accum += time.delta_seconds();
// //                     }
// //                 }
// //
// //                 if timer.finished() {
// //                     if let Some(delete) = delete {
// //                         if !force_update {
// //                             Self::try_play_delete(
// //                                 &self
// //                                     .section_accumulator
// //                                     .iter()
// //                                     .flat_map(|a| a.chars())
// //                                     .collect::<String>(),
// //                                 &mut self.delete_accum,
// //                                 delete,
// //                                 commands,
// //                             );
// //                         }
// //                     }
// //
// //                     if let Some(section) = text.sections.last_mut() {
// //                         section.value.pop();
// //                         self.section_accumulator.last_mut().unwrap().pop();
// //                         *amount -= 1;
// //                     } else {
// //                         warn!("tried to delete from section that does not exist");
// //                     }
// //                 }
// //
// //                 if *amount == 0 {
// //                     end_event = self.fragment_id;
// //                     Some(State::Ready)
// //                 } else {
// //                     None
// //                 }
// //             }
// //             State::Command(command) => match command {
// //                 TextCommand::Speed(speed) => {
// //                     self.chars_per_sec = *speed;
// //                     end_event = self.fragment_id;
// //                     Some(State::Ready)
// //                 }
// //                 TextCommand::Pause(duration) => Some(State::Paused(Timer::new(
// //                     Duration::from_secs_f32(*duration),
// //                     TimerMode::Once,
// //                 ))),
// //                 TextCommand::Clear => {
// //                     self.clear(text);
// //                     end_event = self.fragment_id;
// //                     Some(State::Ready)
// //                 }
// //                 TextCommand::AwaitClear => Some(State::AwaitClear),
// //                 TextCommand::ClearAfter(duration) => Some(State::ClearAfter(Timer::new(
// //                     Duration::from_secs_f32(*duration),
// //                     TimerMode::Once,
// //                 ))),
// //                 TextCommand::Delete(amount) => Some(State::Delete {
// //                     amount: *amount,
// //                     timer: Timer::new(
// //                         Duration::from_secs_f32(1.0 / self.chars_per_sec),
// //                         TimerMode::Repeating,
// //                     ),
// //                 }),
// //             },
// //             State::ClearAfter(timer) => {
// //                 timer.tick(time.delta());
// //                 timer.finished().then(|| {
// //                     self.clear(text);
// //                     end_event = self.fragment_id;
// //                     State::Ready
// //                 })
// //             }
// //             State::AwaitClear => received_input.then(|| {
// //                 self.clear(text);
// //                 end_event = self.fragment_id;
// //                 State::Ready
// //             }),
// //             State::Paused(duration) => {
// //                 duration.tick(time.delta());
// //                 duration.finished().then(|| {
// //                     end_event = self.fragment_id;
// //                     State::Ready
// //                 })
// //             }
// //             State::Sequence {
// //                 sequence,
// //                 type_writer,
// //                 index,
// //                 force_update,
// //             } => {
// //                 let finished = type_writer.finished() && *index >= sequence.len();
// //
// //                 let mut must_render = false;
// //                 if received_input && !*force_update && !finished {
// //                     *force_update = true;
// //
// //                     loop {
// //                         Self::update_seq_type_writer(
// //                             type_writer,
// //                             time,
// //                             false,
// //                             text,
// //                             box_font,
// //                             index,
// //                             sequence,
// //                             commands,
// //                             reveal,
// //                             delete,
// //                             true,
// //                         );
// //
// //                         if type_writer.finished() && *index >= sequence.len() {
// //                             must_render = true;
// //                             break;
// //                         }
// //                     }
// //                 }
// //
// //                 if !must_render
// //                     && *index >= sequence.len()
// //                     && matches!(type_writer.state, State::Ready)
// //                 {
// //                     received_input.then(|| {
// //                         self.clear(text);
// //                         end_event = self.fragment_id;
// //                         State::Ready
// //                     })
// //                 } else {
// //                     Self::update_seq_type_writer(
// //                         type_writer,
// //                         time,
// //                         false,
// //                         text,
// //                         box_font,
// //                         index,
// //                         sequence,
// //                         commands,
// //                         reveal,
// //                         delete,
// //                         false,
// //                     );
// //                     None
// //                 }
// //             }
// //         };
// //
// //         if let Some(new_state) = new_state {
// //             self.state = new_state;
// //         }
// //
// //         end_event.map(|id| id.end())
// //     }
// //
// //     pub fn update_reveal_sfx(
// //         &mut self,
// //         time: &Time,
// //         reveal: AudioBundle,
// //         rate: f32,
// //         commands: &mut Commands,
// //     ) {
// //         match &mut self.state {
// //             State::Section { section, .. } => {
// //                 if section
// //                     .current_section()
// //                     .text
// //                     .chars()
// //                     .last()
// //                     .is_some_and(|c| c != ' ')
// //                 {
// //                     self.reveal_accum -= time.delta_seconds();
// //                     if self.reveal_accum <= 0.0 {
// //                         commands.spawn(reveal);
// //                         self.reveal_accum = rate;
// //                     }
// //                 }
// //             }
// //             State::Sequence { type_writer, .. } => {
// //                 type_writer.update_reveal_sfx(time, reveal, rate, commands);
// //             }
// //             _ => {}
// //         }
// //     }
// //
// //     pub fn update_delete_sfx(
// //         &mut self,
// //         time: &Time,
// //         text: &Text,
// //         delete: AudioBundle,
// //         rate: f32,
// //         commands: &mut Commands,
// //     ) {
// //         match &mut self.state {
// //             State::Delete { .. } => {
// //                 if text
// //                     .sections
// //                     .last()
// //                     .is_some_and(|s| s.value.chars().last().is_some_and(|c| c != ' '))
// //                 {
// //                     self.delete_accum -= time.delta_seconds();
// //                     if self.delete_accum <= 0.0 {
// //                         commands.spawn(delete);
// //                         self.delete_accum = rate;
// //                     }
// //                 }
// //             }
// //             State::Sequence { type_writer, .. } => {
// //                 type_writer.update_delete_sfx(time, text, delete, rate, commands);
// //             }
// //             _ => {}
// //         }
// //     }
// //
// //     pub fn effect_mapping(&self) -> Vec<Option<bevy_bits::TextEffect>> {
// //         match &self.state {
// //             State::Sequence { type_writer, .. } => {
// //                 let mut effects = type_writer.effect_mapping();
// //                 effects.extend(self.effect_mapping.clone());
// //                 effects
// //             }
// //             _ => self.effect_mapping.clone(),
// //         }
// //     }
// //
// //     fn update_seq_type_writer(
// //         type_writer: &mut TypeWriterState,
// //         time: &Time,
// //         received_input: bool,
// //         text: &mut Text,
// //         box_font: &Font,
// //         index: &mut usize,
// //         sequence: &mut Cow<'static, [DialogueBoxToken]>,
// //         commands: &mut Commands,
// //         reveal: Option<&super::audio::RevealedTextSfx>,
// //         delete: Option<&super::audio::DeletedTextSfx>,
// //         force_update: bool,
// //     ) {
// //         if type_writer
// //             .tick(
// //                 time,
// //                 received_input,
// //                 text,
// //                 box_font,
// //                 commands,
// //                 reveal,
// //                 delete,
// //                 force_update,
// //             )
// //             .is_some()
// //             && *index < sequence.len()
// //         {
// //             match sequence[*index].clone() {
// //                 DialogueBoxToken::Section(sec) => {
// //                     type_writer.push_section(sec, Some(FragmentId::random()))
// //                 }
// //                 DialogueBoxToken::Command(cmd) => {
// //                     type_writer.push_cmd(cmd, Some(FragmentId::random()))
// //                 }
// //                 DialogueBoxToken::Sequence(seq) => {
// //                     type_writer.push_seq(seq, Some(FragmentId::random()))
// //                 }
// //             }
// //
// //             *index += 1;
// //         }
// //     }
// //
// //     fn update_text(
// //         text: &mut Text,
// //         effect_mapping: &mut Vec<Option<bevy_bits::TextEffect>>,
// //         section_accumulator: &mut Vec<String>,
// //         box_font: &Font,
// //         section: SectionOccurance,
// //     ) {
// //         match section {
// //             SectionOccurance::First(section, padding) => {
// //                 effect_mapping.push(section.effect.clone());
// //                 section_accumulator.push(section.text.to_string());
// //                 let mut section = section.clone().bevy_section(
// //                     box_font.font.clone(),
// //                     box_font.font_size,
// //                     box_font.default_color,
// //                 );
// //                 section.style.color.set_alpha(0.0);
// //                 section.value.push_str(&padding);
// //                 text.sections.push(section);
// //             }
// //             SectionOccurance::Repeated(section, padding) => {
// //                 *section_accumulator.last_mut().unwrap() = section.text.to_string();
// //                 let mut s = text.sections.last_mut();
// //                 let s = s.as_mut().unwrap();
// //                 s.value = section.text.into();
// //                 s.value.push_str(&padding);
// //             }
// //             SectionOccurance::End(section) => {
// //                 *section_accumulator.last_mut().unwrap() = section.text.to_string();
// //                 text.sections.last_mut().as_mut().unwrap().value = section.text.into();
// //             }
// //         }
// //     }
// //
// //     fn try_play_reveal(
// //         section_accumulator: &str,
// //         reveal_accumulator: &mut f32,
// //         reveal: &super::audio::RevealedTextSfx,
// //         commands: &mut Commands,
// //     ) {
// //         match reveal.settings.trigger {
// //             super::audio::Trigger::Rate(rate) => {
// //                 if *reveal_accumulator > rate {
// //                     commands.spawn(reveal.bundle());
// //                     *reveal_accumulator -= rate;
// //                 }
// //             }
// //             super::audio::Trigger::OnCharacter => {
// //                 if section_accumulator
// //                     .chars()
// //                     .nth_back(0)
// //                     .is_some_and(|c| c != ' ')
// //                 {
// //                     commands.spawn(reveal.bundle());
// //                 }
// //             }
// //             super::audio::Trigger::OnWord => {
// //                 if section_accumulator
// //                     .chars()
// //                     .nth_back(0)
// //                     .is_some_and(|c| c != ' ')
// //                     && section_accumulator
// //                         .chars()
// //                         .nth_back(1)
// //                         .is_none_or(|c| c == ' ')
// //                 {
// //                     commands.spawn(reveal.bundle());
// //                 }
// //             }
// //         }
// //     }
// //
// //     fn try_play_delete(
// //         section_accumulator: &str,
// //         delete_accumulator: &mut f32,
// //         delete: &super::audio::DeletedTextSfx,
// //         commands: &mut Commands,
// //     ) {
// //         match delete.settings.trigger {
// //             super::audio::Trigger::Rate(rate) => {
// //                 if *delete_accumulator > rate {
// //                     commands.spawn(delete.bundle());
// //                     *delete_accumulator -= rate;
// //                 }
// //             }
// //             super::audio::Trigger::OnCharacter => {
// //                 if section_accumulator
// //                     .chars()
// //                     .nth_back(0)
// //                     .is_some_and(|c| c != ' ')
// //                 {
// //                     commands.spawn(delete.bundle());
// //                 }
// //             }
// //             super::audio::Trigger::OnWord => {
// //                 if section_accumulator
// //                     .chars()
// //                     .nth_back(0)
// //                     .is_some_and(|c| c != ' ')
// //                     && section_accumulator
// //                         .chars()
// //                         .nth_back(1)
// //                         .is_none_or(|c| c == ' ')
// //                 {
// //                     commands.spawn(delete.bundle());
// //                 }
// //             }
// //         }
// //     }
// //
// //     fn clear(&mut self, text: &mut Text) {
// //         text.sections.clear();
// //         self.effect_mapping.clear();
// //         self.section_accumulator.clear();
// //     }
// //
// //     fn finished(&self) -> bool {
// //         matches!(self.state, State::Ready)
// //     }
// // }
// //
// // #[derive(Debug, Clone)]
// // enum State {
// //     Ready,
// //     Command(TextCommand),
// //     Section {
// //         section: TypeWriterSectionBuffer,
// //         timer: Timer,
// //     },
// //     Paused(Timer),
// //     AwaitClear,
// //     ClearAfter(Timer),
// //     Delete {
// //         amount: usize,
// //         timer: Timer,
// //     },
// //     Sequence {
// //         sequence: Cow<'static, [bevy_bits::DialogueBoxToken]>,
// //         index: usize,
// //         type_writer: Box<TypeWriterState>,
// //         force_update: bool,
// //     },
// // }
// //
// // #[derive(Component, Debug, Clone)]
// // struct TypeWriterSectionBuffer {
// //     section: bevy_bits::tokens::TextSection,
// //     state: SectionBufferState,
// // }
// //
// // pub enum SectionOccurance {
// //     First(bevy_bits::tokens::TextSection, String),
// //     Repeated(bevy_bits::tokens::TextSection, String),
// //     End(bevy_bits::tokens::TextSection),
// // }
// //
// // #[derive(Debug, Clone)]
// // enum SectionBufferState {
// //     First,
// //     Repeated(usize),
// //     End,
// // }
// //
// // impl TypeWriterSectionBuffer {
// //     pub fn new(section: bevy_bits::tokens::TextSection) -> Self {
// //         Self {
// //             state: SectionBufferState::First,
// //             section,
// //         }
// //     }
// //
// //     pub fn current_section(&self) -> bevy_bits::tokens::TextSection {
// //         match &self.state {
// //             SectionBufferState::First => bevy_bits::tokens::TextSection {
// //                 color: self.section.color.clone(),
// //                 effect: self.section.effect,
// //                 text: Cow::Owned(self.section.text[..1].to_string()),
// //             },
// //             SectionBufferState::Repeated(index) => bevy_bits::tokens::TextSection {
// //                 color: self.section.color.clone(),
// //                 effect: self.section.effect,
// //                 text: Cow::Owned(self.section.text[..*index].to_owned()),
// //             },
// //             SectionBufferState::End => self.section.clone(),
// //         }
// //     }
// //
// //     pub fn advance(&mut self) -> SectionOccurance {
// //         let section = match &mut self.state {
// //             SectionBufferState::First => {
// //                 let space = self
// //                     .section
// //                     .text
// //                     .find(" ")
// //                     .unwrap_or(self.section.text.len() - 1);
// //                 let mut padding = String::with_capacity(space);
// //                 for _ in 0..space {
// //                     padding.push(' ');
// //                 }
// //
// //                 SectionOccurance::First(
// //                     bevy_bits::tokens::TextSection {
// //                         color: self.section.color.clone(),
// //                         effect: self.section.effect,
// //                         text: Cow::Owned(self.section.text[..1].to_owned()),
// //                     },
// //                     padding,
// //                 )
// //             }
// //             SectionBufferState::Repeated(index) => {
// //                 *index += 1;
// //
// //                 let padding = if self.section.text.as_bytes()[index.saturating_sub(1)] != b' ' {
// //                     let mut buf = String::with_capacity(*index);
// //                     if let Some(space) = self.section.text[*index..].find(" ") {
// //                         for _ in 0..space + 1 {
// //                             buf.push(' ');
// //                         }
// //                     } else {
// //                         for _ in *index..self.section.text.len() {
// //                             buf.push(' ');
// //                         }
// //                     }
// //                     buf
// //                 } else {
// //                     String::new()
// //                 };
// //
// //                 SectionOccurance::Repeated(
// //                     bevy_bits::tokens::TextSection {
// //                         color: self.section.color.clone(),
// //                         effect: self.section.effect,
// //                         text: Cow::Owned(self.section.text[..*index].to_owned()),
// //                     },
// //                     padding,
// //                 )
// //             }
// //             SectionBufferState::End => SectionOccurance::End(self.section.clone()),
// //         };
// //
// //         match self.state {
// //             SectionBufferState::First => {
// //                 if self.section.text.len() == 1 {
// //                     self.state = SectionBufferState::End;
// //                 } else {
// //                     self.state = SectionBufferState::Repeated(1);
// //                 }
// //             }
// //             SectionBufferState::Repeated(index) => {
// //                 if self.section.text.len() == index {
// //                     self.state = SectionBufferState::End;
// //                 }
// //             }
// //             _ => {}
// //         }
// //
// //         section
// //     }
// //
// //     pub fn finished(&self) -> bool {
// //         matches!(self.state, SectionBufferState::End { .. })
// //     }
// // }
