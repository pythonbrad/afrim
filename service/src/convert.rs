#![deny(missing_docs)]
//! Set of tools to convert external event keyboards to
//! generic keyboard events and vice versa.
//!

use afrim_preprocessor::{Key, KeyState, KeyboardEvent, NamedKey::*};
use rdev::{self};

/// Converts an rdev::Event into a KeyboardEvent.
pub fn from_event(event: &rdev::Event) -> KeyboardEvent {
    let key_char = event
        .name
        .as_ref()
        .and_then(|c| c.chars().next())
        .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
        .map(|c| Key::Character(c.to_string()));
    let (state, key) = match event.event_type {
        rdev::EventType::KeyPress(key) => (KeyState::Down, from_key(&key)),
        rdev::EventType::KeyRelease(key) => (KeyState::Up, from_key(&key)),
        _ => Default::default(),
    };

    KeyboardEvent {
        key: key_char.unwrap_or(key),
        state,
        ..Default::default()
    }
}

/// Converts an rdev::Key into a Key.
pub fn from_key(key: &rdev::Key) -> Key {
    match key {
        rdev::Key::Alt => Key::Named(Alt),
        rdev::Key::AltGr => Key::Named(AltGraph),
        rdev::Key::Backspace => Key::Named(Backspace),
        rdev::Key::CapsLock => Key::Named(CapsLock),
        rdev::Key::ControlLeft => Key::Named(Control),
        rdev::Key::ControlRight => Key::Named(Control),
        rdev::Key::ShiftLeft => Key::Named(Shift),
        rdev::Key::ShiftRight => Key::Named(Shift),
        rdev::Key::ScrollLock => Key::Named(ScrollLock),
        rdev::Key::Pause => Key::Named(Pause),
        rdev::Key::NumLock => Key::Named(NumLock),
        rdev::Key::Insert => Key::Named(Insert),
        _ => Default::default(),
    }
}
