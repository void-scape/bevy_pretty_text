use bevy::prelude::*;

/// Emitted when a character sound should play.
#[derive(Debug, Event)]
pub struct CharacterEvent;

/// Emitted when a word sound should play.
#[derive(Debug, Event)]
pub struct WordEvent;
