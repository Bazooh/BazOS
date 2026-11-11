pub const KEY_EVENTS_PATH: &str = "key_events.x";

pub const KEY_EVENT_SIZE: usize = 2;

macro_rules! keys {
    (
        ignored: [$($ignored:ident),* $(,)?];
        $($name:ident = $value:literal,)*
    ) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        #[repr(u8)]
        pub enum Key {
            $($name = $value,)*
        }

        impl TryFrom<u8> for Key {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Key::$name),)*
                    _ => Err(()),
                }
            }
        }

        #[cfg(feature = "pc-keyboard")]
        impl TryFrom<pc_keyboard::KeyCode> for Key {
            type Error = ();

            fn try_from(code: pc_keyboard::KeyCode) -> Result<Self, Self::Error> {
                match code {
                    $(pc_keyboard::KeyCode::$name => Ok(Key::$name),)*
                    $(pc_keyboard::KeyCode::$ignored => Err(()),)*
                }
            }
        }
    };
}

keys! {
    // pc-keyboard codes that are not real keys: controller status codes and
    // the fake keys of multi-byte sequences
    ignored: [PowerOnTestOk, TooManyKeys, RControl2, RAlt2];

    // Row 1 (the F-keys)
    Escape = 0,
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
    F5 = 5,
    F6 = 6,
    F7 = 7,
    F8 = 8,
    F9 = 9,
    F10 = 10,
    F11 = 11,
    F12 = 12,
    PrintScreen = 13,
    SysRq = 14,
    ScrollLock = 15,
    PauseBreak = 16,

    // Row 2 (the numbers)
    Oem8 = 17,
    Key1 = 18,
    Key2 = 19,
    Key3 = 20,
    Key4 = 21,
    Key5 = 22,
    Key6 = 23,
    Key7 = 24,
    Key8 = 25,
    Key9 = 26,
    Key0 = 27,
    OemMinus = 28,
    OemPlus = 29,
    Backspace = 30,
    Insert = 31,
    Home = 32,
    PageUp = 33,
    NumpadLock = 34,
    NumpadDivide = 35,
    NumpadMultiply = 36,
    NumpadSubtract = 37,

    // Row 3 (QWERTY)
    Tab = 38,
    Q = 39,
    W = 40,
    E = 41,
    R = 42,
    T = 43,
    Y = 44,
    U = 45,
    I = 46,
    O = 47,
    P = 48,
    Oem4 = 49,
    Oem6 = 50,
    Oem5 = 51,
    Oem7 = 52,
    Delete = 53,
    End = 54,
    PageDown = 55,
    Numpad7 = 56,
    Numpad8 = 57,
    Numpad9 = 58,
    NumpadAdd = 59,

    // Row 4 (ASDF)
    CapsLock = 60,
    A = 61,
    S = 62,
    D = 63,
    F = 64,
    G = 65,
    H = 66,
    J = 67,
    K = 68,
    L = 69,
    Oem1 = 70,
    Oem3 = 71,
    Return = 72,
    Numpad4 = 73,
    Numpad5 = 74,
    Numpad6 = 75,

    // Row 5 (ZXCV)
    LShift = 76,
    Z = 77,
    X = 78,
    C = 79,
    V = 80,
    B = 81,
    N = 82,
    M = 83,
    OemComma = 84,
    OemPeriod = 85,
    Oem2 = 86,
    RShift = 87,
    ArrowUp = 88,
    Numpad1 = 89,
    Numpad2 = 90,
    Numpad3 = 91,
    NumpadEnter = 92,

    // Row 6 (modifiers and space bar)
    LControl = 93,
    LWin = 94,
    LAlt = 95,
    Spacebar = 96,
    RAltGr = 97,
    RWin = 98,
    Apps = 99,
    RControl = 100,
    ArrowLeft = 101,
    ArrowDown = 102,
    ArrowRight = 103,
    Numpad0 = 104,
    NumpadPeriod = 105,

    // JIS 109-key extra keys
    Oem9 = 106,
    Oem10 = 107,
    Oem11 = 108,
    Oem12 = 109,
    Oem13 = 110,

    // Media keys
    PrevTrack = 111,
    NextTrack = 112,
    Mute = 113,
    Calculator = 114,
    Play = 115,
    Stop = 116,
    VolumeDown = 117,
    VolumeUp = 118,
    WWWHome = 119,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum KeyState {
    Up = 0,
    Down = 1,
    /// Pressed and released as one atomic action (e.g. PrintScreen on some keyboards).
    SingleShot = 2,
}

#[cfg(feature = "pc-keyboard")]
impl From<pc_keyboard::KeyState> for KeyState {
    fn from(state: pc_keyboard::KeyState) -> Self {
        match state {
            pc_keyboard::KeyState::Up => KeyState::Up,
            pc_keyboard::KeyState::Down => KeyState::Down,
            pc_keyboard::KeyState::SingleShot => KeyState::SingleShot,
        }
    }
}

impl TryFrom<u8> for KeyState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyState::Up),
            1 => Ok(KeyState::Down),
            2 => Ok(KeyState::SingleShot),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyEvent {
    pub key: Key,
    pub state: KeyState,
}

#[cfg(feature = "pc-keyboard")]
impl TryFrom<pc_keyboard::KeyEvent> for KeyEvent {
    type Error = ();

    fn try_from(event: pc_keyboard::KeyEvent) -> Result<Self, Self::Error> {
        Ok(KeyEvent {
            key: Key::try_from(event.code)?,
            state: KeyState::from(event.state),
        })
    }
}

impl KeyEvent {
    pub fn to_bytes(self) -> [u8; KEY_EVENT_SIZE] {
        [self.key as u8, self.state as u8]
    }

    pub fn from_bytes(bytes: [u8; KEY_EVENT_SIZE]) -> Result<KeyEvent, ()> {
        Ok(KeyEvent {
            key: Key::try_from(bytes[0])?,
            state: KeyState::try_from(bytes[1])?,
        })
    }

    pub fn decode_all(bytes: &[u8]) -> impl Iterator<Item = KeyEvent> + '_ {
        bytes
            .chunks_exact(KEY_EVENT_SIZE)
            .filter_map(|chunk| KeyEvent::from_bytes([chunk[0], chunk[1]]).ok())
    }
}
