use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

/// Number of stars on the session screen.
/// Kept as a constant so it can later become a function of hours worked.
pub const STAR_COUNT: usize = 14;

const FRAME_MS: u128 = 100; // matches the app's 10 fps frame budget
const STAR_CYCLE: [char; 6] = [' ', '.', '+', '*', '+', '.'];
const LEVEL: [usize; 6] = [0, 1, 2, 3, 2, 1];
/// Share of a star's period spent pulsing; the rest it rests (dim or blank).
const PULSE_SHARE: f32 = 0.6;

const STAR_COLORS: [Color; 4] = [
    Color::Rgb(44, 51, 82),
    Color::Rgb(86, 96, 138),
    Color::Rgb(154, 166, 214),
    Color::Rgb(242, 243, 255),
];
const WARM_COLORS: [Color; 2] = [Color::Rgb(201, 168, 120), Color::Rgb(255, 212, 154)];

#[derive(Debug)]
struct Star {
    // position as a fraction of the area, so stars survive resizes
    x: f32,
    y: f32,
    period: u32, // frames per full cycle
    phase: f32,
    warm: bool,
    rest: bool, // hold a dim dot between pulses instead of disappearing
}

/// Twinkling (non-moving) stars drawn into the empty cells of a screen.
#[derive(Debug)]
pub struct Starfield {
    stars: Vec<Star>,
    born: Instant,
}

/// Minimal xorshift, enough to scatter stars without pulling in `rand`.
struct Rng(u64);

impl Rng {
    fn seeded() -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        Self(nanos | 1)
    }

    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
}

impl Starfield {
    pub fn new(count: usize) -> Self {
        let mut rng = Rng::seeded();
        let stars = (0..count)
            .map(|_| Star {
                x: rng.next_f32(),
                y: rng.next_f32(),
                period: 14 + (rng.next_f32() * 30.0) as u32,
                phase: rng.next_f32(),
                warm: rng.next_f32() < 0.18,
                rest: rng.next_f32() < 0.5,
            })
            .collect();
        Self { stars, born: Instant::now() }
    }
}

impl Widget for &Starfield {
    /// Only paints blank cells, so render it after the screen's content.
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() { return; }
        let frame = (self.born.elapsed().as_millis() / FRAME_MS) as f32;

        for star in &self.stars {
            let t = (frame / star.period as f32 + star.phase).fract();
            let step = if t < PULSE_SHARE {
                ((t / PULSE_SHARE) * STAR_CYCLE.len() as f32) as usize
            } else if star.rest { 1 } else { 0 };
            let step = step.min(STAR_CYCLE.len() - 1);
            if step == 0 { continue; }

            let x = area.x + (star.x * area.width as f32) as u16 % area.width;
            let y = area.y + (star.y * area.height as f32) as u16 % area.height;
            let Some(cell) = buf.cell_mut((x, y)) else { continue };
            if cell.symbol() != " " { continue; }

            let level = LEVEL[step];
            let mut style = Style::new().fg(match level {
                2 | 3 if star.warm => WARM_COLORS[level - 2],
                _ => STAR_COLORS[level],
            });
            if level == 3 { style = style.add_modifier(Modifier::BOLD); }
            cell.set_char(STAR_CYCLE[step]).set_style(style);
        }
    }
}
