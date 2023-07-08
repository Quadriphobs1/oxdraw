use smol_str::SmolStr;
use std::fmt::Display;

use winit::keyboard;

use crate::event::KeyboardModifiers;

/// Code is the physical position of a key.
///
/// The names are based on the US keyboard. If the key
/// is not present on US keyboards a name from another
/// layout is used.
///
/// Specification:
/// <https://w3c.github.io/uievents-code/>
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum KeyCode {
    // TODO(Quadri): Handle character codes for individual keys
    /// `abcdefghijklmnopqrstuvwxyz1234567890` key on any standard keyboard.
    Character(SmolStr, bool),
    /// `Alt`, `Option` or `⌥`.
    AltOrOption,
    /// `Backspace`or `⌫`.
    /// Labelled `Delete` on Apple keyboards.
    BackspaceOrDelete,
    /// `CapsLock`or `⇪`
    CapsLock,
    /// `Control`or `⌃`
    Control,
    /// `Enter`or `↵` Labelled `Return` on Apple keyboards.
    Enter,
    /// The Windows, `⌘`, `Command` or other OS symbol key.
    Command,
    /// `Shift` or `⇧`
    Shift,
    /// ` ` (space)
    Space,
    /// `Tab`or `⇥`
    Tab,
    /// `⌦`. The forward delete key.
    /// Note that on Apple keyboards, the key labelled `Delete` on the main part of
    /// the keyboard should be encoded as `"Backspace"`.
    FDelete,
    /// `↓`
    ArrowDown,
    /// `←`
    ArrowLeft,
    /// `→`
    ArrowRight,
    /// `↑`
    ArrowUp,
    /// `Esc`or `⎋`
    Escape,
    /// `Home`
    Home,
    /// `End`
    End,
    /// `PageDown`
    PageDown,
    /// `PageUp`
    PageUp,
    NonConvert,
}

impl Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::KeyCode::{
            AltOrOption, ArrowDown, ArrowLeft, ArrowRight, ArrowUp, BackspaceOrDelete, CapsLock,
            Character, Command, Control, End, Enter, Escape, FDelete, Home, NonConvert, PageDown,
            PageUp, Shift, Space, Tab,
        };
        match self {
            Character(c, shift) => f.write_str(format!("{c} with shift {shift}").as_str()),
            AltOrOption => f.write_str("Alt"),
            BackspaceOrDelete => f.write_str("Backspace"),
            CapsLock => f.write_str("CapsLock"),
            Control => f.write_str("Control"),
            Enter => f.write_str("Enter"),
            Command => f.write_str("Command"),
            Shift => f.write_str("Shift"),
            Space => f.write_str(" "),
            Tab => f.write_str("Tab"),
            FDelete => f.write_str("ForwardDelete"),
            ArrowDown => f.write_str("ArrowDown"),
            ArrowLeft => f.write_str("ArrowLeft"),
            ArrowRight => f.write_str("ArrowRight"),
            ArrowUp => f.write_str("ArrowUp"),
            Escape => f.write_str("Escape"),
            Home => f.write_str("Home"),
            End => f.write_str("End"),
            PageDown => f.write_str("PageDown"),
            PageUp => f.write_str("PageUp"),
            NonConvert => f.write_str("NonConvert"),
        }
    }
}

pub fn key_event_to_code(key: keyboard::Key, key_mods: &KeyboardModifiers) -> KeyCode {
    match key {
        keyboard::Key::Character(c) => KeyCode::Character(c, key_mods.shift),
        keyboard::Key::Alt => KeyCode::AltOrOption,
        keyboard::Key::CapsLock => KeyCode::CapsLock,
        keyboard::Key::Control => KeyCode::Control,
        keyboard::Key::Shift => KeyCode::Shift,
        keyboard::Key::Meta | keyboard::Key::Super => KeyCode::Command,
        keyboard::Key::Enter => KeyCode::Enter,
        keyboard::Key::Tab => KeyCode::Tab,
        keyboard::Key::Space => KeyCode::Space,
        keyboard::Key::ArrowDown => KeyCode::ArrowDown,
        keyboard::Key::ArrowLeft => KeyCode::ArrowLeft,
        keyboard::Key::ArrowRight => KeyCode::ArrowRight,
        keyboard::Key::ArrowUp => KeyCode::ArrowUp,
        keyboard::Key::End => KeyCode::End,
        keyboard::Key::Home => KeyCode::Home,
        keyboard::Key::PageDown => KeyCode::PageDown,
        keyboard::Key::PageUp => KeyCode::PageUp,
        keyboard::Key::Backspace => KeyCode::BackspaceOrDelete,
        keyboard::Key::Delete => KeyCode::FDelete,
        keyboard::Key::Escape => KeyCode::Escape,
        _ => KeyCode::NonConvert,
    }
}
