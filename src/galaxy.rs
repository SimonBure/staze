use std::f32::consts::{PI, TAU};
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::appearance;
use crate::starfield::{put, Rng};

/// Twinkling stars scattered around the galaxy.
pub const FIELD_STAR_COUNT: usize = 9;

const MAX_RADIUS_X: f32 = 70.0; // cells
/// Particles per cell of the galaxy's ellipse, so it looks equally dense at any size.
const DENSITY: f32 = 3.0;
const PARTICLE_BOUNDS: (usize, usize) = (200, 40_000);
/// Cell height / width: a terminal cell is about twice as tall as it is wide.
const CELL_ASPECT: f32 = 0.5;
const SEED: u64 = 0x5747_A2E5;

// Same glyphs as the session starfield: density picks the step, colour carries the finer gradient.
const GLYPHS: [char; 3] = ['.', '+', '*'];
const CORE_RADIUS_SQ: f32 = 0.045;

/// Sparkle: each cell briefly brightens one step, at its own period (in frames).
const SPARKLE_PERIOD: (u64, u64) = (18, 30);
const SPARKLE_SHARE: f32 = 0.1;

struct Particle {
    r: f32, // 0..1, fraction of the galaxy radius
    a: f32, // angle at rotation 0
    w: f32, // brightness weight
}

/// Slowly rotating spiral galaxy, drawn into the blank cells of an area.
/// Particles are rebuilt only when the size or shape settings change.
#[derive(Default)]
pub struct Galaxy {
    particles: Vec<Particle>,
    built_for: Option<ShapeKey>,
}

/// Everything the particle set depends on: fitted radii and arm settings.
#[derive(PartialEq, Clone, Copy)]
struct ShapeKey {
    rx: u32, // radii in hundredths of a cell
    ry: u32,
    arms: u8,
    winding: u32,
}

impl std::fmt::Debug for Galaxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Galaxy").field("particles", &self.particles.len()).finish()
    }
}

/// Largest ellipse with the galaxy's natural proportions (set by the tilt) that fits `area`.
fn fit(area: Rect, tilt: f32) -> (f32, f32) {
    let ratio = CELL_ASPECT * tilt; // ry / rx
    let rx = (area.width as f32 / 2.0)
        .min(area.height as f32 / 2.0 / ratio)
        .min(MAX_RADIUS_X);
    (rx, rx * ratio)
}

impl Galaxy {
    /// The seed is fixed, so the same settings and size always give the same galaxy.
    fn build(arms: u8, winding: f32, count: usize) -> Vec<Particle> {
        let arms = arms as usize;
        let mut rng = Rng(SEED);
        let gauss = |rng: &mut Rng| (rng.next_f32() + rng.next_f32() + rng.next_f32() - 1.5) / 1.5;
        (0..count).map(|_| {
            let kind = rng.next_f32();
            if kind < 0.22 {
                // bulge
                Particle { r: gauss(&mut rng).abs() * 0.16, a: rng.next_f32() * TAU, w: 1.4 }
            } else if kind < 0.34 {
                // diffuse disk
                Particle { r: 0.1 + rng.next_f32() * 0.9, a: rng.next_f32() * TAU, w: 0.35 }
            } else {
                // spiral arms
                let arm = (rng.next_f32() * arms as f32) as usize;
                let r = 0.06 + rng.next_f32().powf(0.9) * 0.94;
                let spread = 0.12 + 0.3 * (1.0 - r);
                let a = arm as f32 * TAU / arms as f32 + winding * PI * r + gauss(&mut rng) * spread;
                Particle { r, a, w: 0.9 * (1.15 - r) + 0.25 }
            }
        }).collect()
    }
}

fn cell_hash(x: u16, y: u16) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

impl Widget for &mut Galaxy {
    /// Centres the galaxy in `area`. Only paints blank cells, so render it after the screen's content.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (w, h) = (area.width as usize, area.height as usize);
        let cx = (w as f32 - 1.0) / 2.0;
        let cy = (h as f32 - 1.0) / 2.0;
        let settings = appearance::get();
        let (rx, ry) = fit(area, settings.tilt);
        if rx < 2.0 || ry < 1.0 { return; }

        let key = ShapeKey {
            rx: (rx * 100.0) as u32,
            ry: (ry * 100.0) as u32,
            arms: settings.arms,
            winding: (settings.winding * 100.0) as u32,
        };
        if self.built_for != Some(key) {
            let count = (DENSITY * PI * rx * ry) as usize;
            self.particles = Galaxy::build(settings.arms, settings.winding, count.clamp(PARTICLE_BOUNDS.0, PARTICLE_BOUNDS.1));
            self.built_for = Some(key);
        }

        // Wall clock, so the rotation carries on across screen changes
        let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let turn_ms = settings.turn_secs as u128 * 1000;
        let rot = (ms % turn_ms) as f32 / turn_ms as f32 * TAU;
        let frame = (ms / 100) as u64;

        let theme = appearance::theme();
        let mut density = vec![0f32; w * h];
        for p in &self.particles {
            let a = p.a - rot;
            let x = (cx + a.cos() * p.r * rx).round();
            let y = (cy + a.sin() * p.r * ry).round();
            if x >= 0.0 && y >= 0.0 && (x as usize) < w && (y as usize) < h {
                density[y as usize * w + x as usize] += p.w;
            }
        }

        for y in 0..h {
            for x in 0..w {
                let d = density[y * w + x];
                if d <= 0.0 { continue; }
                let v = d / (d + 3.2); // soft tone-map so the core doesn't flatten
                let mut glyph = if v < 0.22 { 0 } else if v < 0.5 { 1 } else { 2 };
                let mut level = ((v * 5.5) as usize).min(4);

                let hash = cell_hash(area.x + x as u16, area.y + y as u16);
                let period = SPARKLE_PERIOD.0 + (hash * SPARKLE_PERIOD.1 as f32) as u64;
                if settings.sparkle && ((frame as f32 / period as f32) + hash * 7.0).fract() < SPARKLE_SHARE {
                    glyph = (glyph + 1).min(2);
                    level = (level + 1).min(4);
                }

                let (dx, dy) = ((x as f32 - cx) / rx, (y as f32 - cy) / ry);
                let color = if dx * dx + dy * dy < CORE_RADIUS_SQ { theme.core[level] } else { theme.arms[level] };
                let mut style = Style::new().fg(color);
                if level == 4 { style = style.add_modifier(Modifier::BOLD); }
                put(buf, area, x as i32, y as i32, GLYPHS[glyph], style);
            }
        }
    }
}
