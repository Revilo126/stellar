//! Inputs for keyboards

use std::fmt;

use fent_ecs::prelude::*;

#[derive(Message)]
pub enum KeyEvent {
    KeyDown {
        window: Entity,
        key_code: Option<KeyCode>,
        repeat: bool,
    },
    KeyUp {
        window: Entity,
        key_code: Option<KeyCode>,
        repeat: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,
    NumpadEqual,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Fn,

    Escape,
    Delete,
    Home,
    End,
    Insert,
    Backspace,
    Tab,
    Space,
    CapsLock,
    NumLock,
    ScrollLock,
    MetaLeft,
    MetaRight,
    ContextMenu,
    PrintScreen,
    Pause,

    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Period,
    Colon,
    Semicolon,
    Exclaim,
    Asterix,
    Question,
    Minus,
    Plus,
    Quote,
    Slash,

    Less,
    Equal,
    Greater,

    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,

    Right,
    Left,
    Up,
    Down,
    PageUp,
    PageDown,

    IntlBackslash,
    IntlRo,
    IntlYen,
    KanaMode,
    Convert,
    NonConvert,

    AudioVolumeUp,
    AudioVolumeDown,
    AudioVolumeMute,
    MediaPlayPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,

    Unidentified,
    Scancode(u32),
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl KeyCode {
    pub fn name(&self) -> String {
        format!("{self:?}")
    }
}
