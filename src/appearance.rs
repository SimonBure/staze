//! Look-and-feel settings read at render time: the active theme, which skies are shown and how the galaxy looks.
//! Set by `App` from the config; screens and sky widgets only read them.

use std::sync::RwLock;

use ratatui::style::Color;

pub struct Theme {
    pub key: &'static str,
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub timer: Color,
    pub stars: [Color; 4],
    pub warm: [Color; 2],
    pub meteor_head: Color,
    pub meteor_tail: Color,
    pub arms: [Color; 5],
    pub core: [Color; 5],
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub const THEMES: [Theme; 4] = [
    Theme {
        key: "dark-blue", name: "Dark · Blue",
        bg: rgb(0x03040b), fg: rgb(0xc9cde4), timer: rgb(0x7ee2a8),
        stars: [rgb(0x2c3352), rgb(0x56608a), rgb(0x9aa6d6), rgb(0xf2f3ff)],
        warm: [rgb(0xc9a878), rgb(0xffd49a)],
        meteor_head: rgb(0xdfe6ff), meteor_tail: rgb(0x4b5687),
        arms: [rgb(0x262d4c), rgb(0x3f4a7c), rgb(0x6878b8), rgb(0xa9b6ec), rgb(0xeef1ff)],
        core: [rgb(0x6b5a52), rgb(0x6b5a52), rgb(0xb08d6c), rgb(0xe8c190), rgb(0xfff1d6)],
    },
    Theme {
        key: "dark-red", name: "Dark · Red",
        bg: rgb(0x0b0405), fg: rgb(0xe4cfd0), timer: rgb(0xffb870),
        stars: [rgb(0x3a1d22), rgb(0x6e3038), rgb(0xc8616b), rgb(0xffe6e0)],
        warm: [rgb(0xc99a6a), rgb(0xffcf8a)],
        meteor_head: rgb(0xffd9d2), meteor_tail: rgb(0x6a3038),
        arms: [rgb(0x2e1519), rgb(0x572530), rgb(0x9a414d), rgb(0xdf7680), rgb(0xffe3df)],
        core: [rgb(0x6b4a3a), rgb(0x6b4a3a), rgb(0xb8785a), rgb(0xf0ae7a), rgb(0xfff0dc)],
    },
    Theme {
        key: "light-blue", name: "Light · Blue",
        bg: rgb(0xf4f6fb), fg: rgb(0x2a2f45), timer: rgb(0x1f8a55),
        stars: [rgb(0xd5dbea), rgb(0xa3aed0), rgb(0x5a6aa6), rgb(0x1c2658)],
        warm: [rgb(0xb98a4a), rgb(0x8a5a14)],
        meteor_head: rgb(0x1c2658), meteor_tail: rgb(0xa3aed0),
        arms: [rgb(0xdde2f0), rgb(0xb3bddb), rgb(0x7f8fc2), rgb(0x44559a), rgb(0x141d4d)],
        core: [rgb(0xd8c6b0), rgb(0xd8c6b0), rgb(0xc09a6a), rgb(0x9a6a2a), rgb(0x5e3a06)],
    },
    Theme {
        key: "light-red", name: "Light · Red",
        bg: rgb(0xfbf4f3), fg: rgb(0x3a2a2d), timer: rgb(0xb8561a),
        stars: [rgb(0xefd6d6), rgb(0xd9a4a8), rgb(0xb0505c), rgb(0x5a121c)],
        warm: [rgb(0xb98a4a), rgb(0x8a5a14)],
        meteor_head: rgb(0x5a121c), meteor_tail: rgb(0xd9a4a8),
        arms: [rgb(0xf2dcdc), rgb(0xe0b0b4), rgb(0xc07078), rgb(0x963440), rgb(0x4a0c16)],
        core: [rgb(0xe3cdb8), rgb(0xe3cdb8), rgb(0xc99b70), rgb(0xa0662c), rgb(0x5e3406)],
    },
];

/// Everything the Settings screen controls.
#[derive(Clone, Copy, Debug)]
pub struct Appearance {
    pub theme: usize,
    pub galaxy: bool,
    pub session_stars: bool,
    pub arms: u8,
    pub turn_secs: u32, // one full galaxy rotation
    pub tilt: f32,      // y squash, on top of the ~0.5 cell aspect ratio
    pub winding: f32,   // arm angle = arm·2π/arms + winding·π·r
    pub sparkle: bool,
    pub field_stars: bool,
}

const DEFAULT: Appearance = Appearance {
    theme: 0,
    galaxy: true,
    session_stars: true,
    arms: 2,
    turn_secs: 240,
    tilt: 0.55,
    winding: 1.6,
    sparkle: true,
    field_stars: true,
};

impl Default for Appearance {
    fn default() -> Self {
        DEFAULT
    }
}

pub const ARMS_RANGE: (u8, u8) = (2, 4);
pub const TURN_SECS_RANGE: (u32, u32, u32) = (30, 600, 30); // min, max, step
pub const TILT_RANGE: (f32, f32, f32) = (0.25, 1.0, 0.05);
pub const WINDING_RANGE: (f32, f32, f32) = (0.6, 3.0, 0.1);

impl Appearance {
    /// Brings values loaded from a hand-edited config back into range.
    pub fn clamped(mut self) -> Self {
        self.theme = self.theme.min(THEMES.len() - 1);
        self.arms = self.arms.clamp(ARMS_RANGE.0, ARMS_RANGE.1);
        self.turn_secs = self.turn_secs.clamp(TURN_SECS_RANGE.0, TURN_SECS_RANGE.1);
        self.tilt = self.tilt.clamp(TILT_RANGE.0, TILT_RANGE.1);
        self.winding = self.winding.clamp(WINDING_RANGE.0, WINDING_RANGE.1);
        self
    }
}

static CURRENT: RwLock<Appearance> = RwLock::new(DEFAULT);

pub fn get() -> Appearance {
    *CURRENT.read().unwrap_or_else(|e| e.into_inner())
}

pub fn set(appearance: Appearance) {
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = appearance.clamped();
}

pub fn theme() -> &'static Theme {
    &THEMES[get().theme]
}

pub fn theme_by_key(key: &str) -> Option<usize> {
    THEMES.iter().position(|t| t.key == key)
}
