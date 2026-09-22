use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::appearance::{self, Appearance, ARMS_RANGE, THEMES, TILT_RANGE, TURN_SECS_RANGE, WINDING_RANGE};

#[derive(Clone, Copy, PartialEq)]
enum Row {
    Theme,
    Galaxy,
    SessionStars,
    Arms,
    Turn,
    Tilt,
    Winding,
    Sparkle,
    FieldStars,
}

const ROWS: [Row; 9] = [
    Row::Theme, Row::Galaxy, Row::SessionStars,
    Row::Arms, Row::Turn, Row::Tilt, Row::Winding, Row::Sparkle, Row::FieldStars,
];
/// Rows from here on tune the galaxy and sit under their own heading.
const FIRST_GALAXY_ROW: usize = 3;

pub struct Settings {
    selected: usize,
}

pub enum SettingsAction {
    None,
    Stop,
    /// A setting changed and is already applied; App should persist it.
    Changed,
}

/// Steps `value` by `step` increments within `(min, max, inc)`, rounded to kill float drift.
fn nudge(value: f32, step: isize, (min, max, inc): (f32, f32, f32)) -> f32 {
    ((value + step as f32 * inc).clamp(min, max) * 100.0).round() / 100.0
}

fn format_turn(secs: u32) -> String {
    match (secs / 60, secs % 60) {
        (0, s) => format!("{s} s"),
        (m, 0) => format!("{m} min"),
        (m, s) => format!("{m} min {s} s"),
    }
}

impl Settings {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// `step` is +1/-1; toggles ignore the direction, lists wrap, ranges clamp.
    fn change(&mut self, step: isize) -> SettingsAction {
        let mut a = appearance::get();
        match ROWS[self.selected] {
            Row::Theme => a.theme = (a.theme as isize + step).rem_euclid(THEMES.len() as isize) as usize,
            Row::Galaxy => a.galaxy = !a.galaxy,
            Row::SessionStars => a.session_stars = !a.session_stars,
            Row::Arms => {
                let (min, max) = ARMS_RANGE;
                let span = (max - min + 1) as isize;
                a.arms = min + (a.arms as isize - min as isize + step).rem_euclid(span) as u8;
            }
            Row::Turn => {
                let (min, max, inc) = TURN_SECS_RANGE;
                a.turn_secs = (a.turn_secs as isize + step * inc as isize).clamp(min as isize, max as isize) as u32;
            }
            Row::Tilt => a.tilt = nudge(a.tilt, step, TILT_RANGE),
            Row::Winding => a.winding = nudge(a.winding, step, WINDING_RANGE),
            Row::Sparkle => a.sparkle = !a.sparkle,
            Row::FieldStars => a.field_stars = !a.field_stars,
        }
        appearance::set(a);
        SettingsAction::Changed
    }

    pub fn handle_key(&mut self, key: KeyCode) -> SettingsAction {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                SettingsAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(ROWS.len() - 1);
                SettingsAction::None
            }
            KeyCode::Left | KeyCode::Char('h') => self.change(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => self.change(1),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => SettingsAction::Stop,
            _ => SettingsAction::None,
        }
    }
}

fn row_text(row: Row, a: &Appearance) -> (&'static str, String) {
    let on_off = |b: bool| format!("[ {} ]", if b { "on" } else { "off" });
    match row {
        Row::Theme => ("Theme", format!("< {} >", THEMES[a.theme].name)),
        Row::Galaxy => ("Home galaxy", on_off(a.galaxy)),
        Row::SessionStars => ("Session stars", on_off(a.session_stars)),
        Row::Arms => ("Arms", format!("< {} >", a.arms)),
        Row::Turn => ("One full turn", format!("< {} >", format_turn(a.turn_secs))),
        Row::Tilt => ("Tilt", format!("< {:.0}% >", a.tilt * 100.0)),
        Row::Winding => ("Arm winding", format!("< {:.1} >", a.winding)),
        Row::Sparkle => ("Sparkle", on_off(a.sparkle)),
        Row::FieldStars => ("Field stars", on_off(a.field_stars)),
    }
}

impl Widget for &Settings {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hint = Line::from(vec![
            " Navigate ".into(),
            "<↑↓> ; <k/j>".blue().bold(),
            "  Change ".into(),
            "<←→> ; <Enter>".blue().bold(),
            "  Back ".into(),
            "<Esc> ".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(Line::from(" Settings ".bold()).centered())
            .title_bottom(hint.centered())
            .border_set(border::THICK);
        let inner = block.inner(area);
        block.render(area, buf);

        let a = appearance::get();
        // Galaxy rows fade out when the galaxy is off, but stay editable
        let faded = Style::new().fg(appearance::theme().stars[1]);

        let mut lines: Vec<Line> = Vec::new();
        for (i, row) in ROWS.iter().enumerate() {
            if i == FIRST_GALAXY_ROW {
                lines.push(Line::default());
                lines.push(Line::from("─── Galaxy ───").bold());
            }
            let (label, value) = row_text(*row, &a);
            let mut style = if i >= FIRST_GALAXY_ROW && !a.galaxy { faded } else { Style::new() };
            if i == self.selected { style = style.reversed(); }
            lines.push(Line::from(format!("{label:<16}{value:>18}")).style(style));
        }

        let [_, list, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(lines.len() as u16),
            Constraint::Fill(1),
        ]).areas(inner);
        Paragraph::new(lines).centered().render(list, buf);
    }
}
