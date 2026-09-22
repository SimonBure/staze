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

// Shooting stars run on their own clock, independent of the star count:
// time is cut into slots, and each slot may hold one short streak.
const METEOR_SLOT_FRAMES: u64 = 50;
const METEOR_CHANCE: f32 = 0.5;
const METEOR_HEAD: (char, Color) = ('*', Color::Rgb(223, 230, 255));
const METEOR_TAIL: (char, Color) = ('\\', Color::Rgb(75, 86, 135));
const METEOR_TAIL_END: (char, Color) = ('.', Color::Rgb(75, 86, 135));

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
    seed: u64,
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
        Self { stars, born: Instant::now(), seed: rng.0 }
    }

    /// Head position and tail length of the shooting star visible at `frame`, if any.
    fn meteor(&self, frame: u64, area: Rect) -> Option<(i32, i32, i32)> {
        let slot = frame / METEOR_SLOT_FRAMES;
        let mut rng = Rng(splitmix(self.seed ^ slot));
        if rng.next_f32() >= METEOR_CHANCE { return None; }
        let start = (rng.next_f32() * 20.0) as u64;
        let life = 6 + (rng.next_f32() * 8.0) as u64;
        let age = (frame % METEOR_SLOT_FRAMES).checked_sub(start).filter(|a| *a < life)? as i32;
        let x0 = (area.width as f32 * (0.3 + rng.next_f32() * 0.7)) as i32;
        let y0 = (area.height as f32 * rng.next_f32() * 0.5) as i32;
        let len = 3 + (rng.next_f32() * 3.0) as i32;
        // travels down-left, one cell per frame
        Some((x0 - age, y0 + age, len.min(age)))
    }
}

fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    (z ^ (z >> 31)) | 1
}

/// Sets `ch` at (x, y) relative to `area` if that cell is inside and blank.
fn put(buf: &mut Buffer, area: Rect, x: i32, y: i32, ch: char, style: Style) {
    if x < 0 || y < 0 || x >= area.width as i32 || y >= area.height as i32 { return; }
    if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y + y as u16))
        && cell.symbol() == " " {
            cell.set_char(ch).set_style(style);
        }
}

impl Widget for &Starfield {
    /// Only paints blank cells, so render it after the screen's content.
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() { return; }
        let frame_n = (self.born.elapsed().as_millis() / FRAME_MS) as u64;
        let frame = frame_n as f32;

        for star in &self.stars {
            let t = (frame / star.period as f32 + star.phase).fract();
            let step = if t < PULSE_SHARE {
                ((t / PULSE_SHARE) * STAR_CYCLE.len() as f32) as usize
            } else if star.rest { 1 } else { 0 };
            let step = step.min(STAR_CYCLE.len() - 1);
            if step == 0 { continue; }

            let x = (star.x * area.width as f32) as i32;
            let y = (star.y * area.height as f32) as i32;
            let level = LEVEL[step];
            let mut style = Style::new().fg(match level {
                2 | 3 if star.warm => WARM_COLORS[level - 2],
                _ => STAR_COLORS[level],
            });
            if level == 3 { style = style.add_modifier(Modifier::BOLD); }
            put(buf, area, x, y, STAR_CYCLE[step], style);
        }

        if let Some((x, y, len)) = self.meteor(frame_n, area) {
            for i in 1..=len {
                let (ch, color) = if i == len { METEOR_TAIL_END } else { METEOR_TAIL };
                put(buf, area, x + i, y - i, ch, Style::new().fg(color));
            }
            put(buf, area, x, y, METEOR_HEAD.0, Style::new().fg(METEOR_HEAD.1).add_modifier(Modifier::BOLD));
        }
    }
}
