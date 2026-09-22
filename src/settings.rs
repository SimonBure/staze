use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::appearance::{self, THEMES};

const ROWS: usize = 3;

pub struct Settings {
    selected: usize,
}

pub enum SettingsAction {
    None,
    Stop,
    /// A setting changed and is already applied; App should persist it.
    Changed,
}

impl Settings {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// `step` is +1/-1 for the theme; toggles ignore the direction.
    fn change(&mut self, step: isize) -> SettingsAction {
        let (mut theme, mut galaxy, mut stars) =
            (appearance::theme_index(), appearance::galaxy(), appearance::session_stars());
        match self.selected {
            0 => theme = (theme as isize + step).rem_euclid(THEMES.len() as isize) as usize,
            1 => galaxy = !galaxy,
            _ => stars = !stars,
        }
        appearance::set(theme, galaxy, stars);
        SettingsAction::Changed
    }

    pub fn handle_key(&mut self, key: KeyCode) -> SettingsAction {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                SettingsAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(ROWS - 1);
                SettingsAction::None
            }
            KeyCode::Left | KeyCode::Char('h') => self.change(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => self.change(1),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => SettingsAction::Stop,
            _ => SettingsAction::None,
        }
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

        let on_off = |b: bool| if b { "on" } else { "off" };
        let rows = [
            ("Theme", format!("< {} >", appearance::theme().name)),
            ("Home galaxy", format!("[ {} ]", on_off(appearance::galaxy()))),
            ("Session stars", format!("[ {} ]", on_off(appearance::session_stars()))),
        ];

        let [_, list, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(ROWS as u16 * 2 - 1),
            Constraint::Fill(1),
        ]).areas(inner);
        for (i, (label, value)) in rows.iter().enumerate() {
            let y = list.y + i as u16 * 2;
            if y >= list.bottom() { break; }
            let style = if i == self.selected { Style::new().reversed() } else { Style::new() };
            let line = Line::from(vec![
                format!("{label:<16}").into(),
                format!("{value:>18}").into(),
            ]).style(style);
            Paragraph::new(line).centered().render(Rect { y, height: 1, ..list }, buf);
        }
    }
}
