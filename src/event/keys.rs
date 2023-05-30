

bitflags::bitflags! {
    /// Represents key modifiers (shift, control, alt).
    pub struct KeyModifiers: u8 {
        #[allow(missing_docs)]
        const SHIFT = 0b0000_0001;
        #[allow(missing_docs)]
        const CONTROL = 0b0000_0010;
        #[allow(missing_docs)]
        const ALT = 0b0000_0100;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct KeyEvent {
    /// The key itself.
    pub code: KeyCode,
    /// Additional key modifiers.
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Creates a new `KeyEvent`
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent { code, modifiers }
    }
}

impl From<KeyCode> for KeyEvent {
    fn from(code: KeyCode) -> Self {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }
}



/// Represents an key.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum KeyCode  {
    /// Both Enter (or Return) and numpad Enter
    Enter,
    /// Tabulation key
    Tab,
    /// Backspace key
    Backspace,
    /// Escape key
    Esc,

    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,

    /// Insert key
    Ins,
    /// Delete key
    Delete,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    F(u8),
    Char(char),
    Null,
}

// impl Key {
//     /// Returns the function key corresponding to the given number
//     ///
//     /// 1 -> F1, etc...
//     ///
//     /// # Panics
//     ///
//     /// If `n == 0 || n > 12`
//     pub fn from_f(n: u8) -> Key {
//         match n {
//             0 => Key::F0,
//             1 => Key::F1,
//             2 => Key::F2,
//             3 => Key::F3,
//             4 => Key::F4,
//             5 => Key::F5,
//             6 => Key::F6,
//             7 => Key::F7,
//             8 => Key::F8,
//             9 => Key::F9,
//             10 => Key::F10,
//             11 => Key::F11,
//             12 => Key::F12,
//             _ => panic!("unknown function key: F{}", n),
//         }
//     }
// }

// impl From<event::KeyEvent> for Key {
//     fn from(key_event: event::KeyEvent) -> Self {
//         match key_event {
//             event::KeyEvent {
//                 code: event::KeyCode::Esc,
//                 ..
//             } => Key::Esc,
//             event::KeyEvent {
//                 code: event::KeyCode::Backspace,
//                 ..
//             } => Key::Backspace,
//             event::KeyEvent {
//                 code: event::KeyCode::Left,
//                 ..
//             } => Key::Left,
//             event::KeyEvent {
//                 code: event::KeyCode::Right,
//                 ..
//             } => Key::Right,
//             event::KeyEvent {
//                 code: event::KeyCode::Up,
//                 ..
//             } => Key::Up,
//             event::KeyEvent {
//                 code: event::KeyCode::Down,
//                 ..
//             } => Key::Down,
//             event::KeyEvent {
//                 code: event::KeyCode::Home,
//                 ..
//             } => Key::Home,
//             event::KeyEvent {
//                 code: event::KeyCode::End,
//                 ..
//             } => Key::End,
//             event::KeyEvent {
//                 code: event::KeyCode::PageUp,
//                 ..
//             } => Key::PageUp,
//             event::KeyEvent {
//                 code: event::KeyCode::PageDown,
//                 ..
//             } => Key::PageDown,
//             event::KeyEvent {
//                 code: event::KeyCode::Delete,
//                 ..
//             } => Key::Delete,
//             event::KeyEvent {
//                 code: event::KeyCode::Insert,
//                 ..
//             } => Key::Ins,
//             event::KeyEvent {
//                 code: event::KeyCode::F(n),
//                 ..
//             } => Key::from_f(n),
//             event::KeyEvent {
//                 code: event::KeyCode::Enter,
//                 ..
//             } => Key::Enter,
//             event::KeyEvent {
//                 code: event::KeyCode::Tab,
//                 ..
//             } => Key::Tab,

//             // First check for char + modifier
//             event::KeyEvent {
//                 code: event::KeyCode::Char(c),
//                 modifiers: event::KeyModifiers::ALT,
//                 ..
//             } => Key::Alt(c),
//             event::KeyEvent {
//                 code: event::KeyCode::Char(c),
//                 modifiers: event::KeyModifiers::CONTROL,
//                 ..
//             } => Key::Ctrl(c),

//             event::KeyEvent {
//                 code: event::KeyCode::Char(c),
//                 ..
//             } => Key::Char(c),

//             _ => Key::Unkown,
//         }
//     }
// }
