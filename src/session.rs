use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct Session {
    pub tag: String,
    is_tag_default: bool,
    editing: bool,
    selected: u8,
    start: Instant,
    started_at: u64,
}

pub enum SessionAction {
    None,
    Stop,
}

impl Session {
    pub fn new() -> Self {
        Self {
            tag: "wonderful-thinking-session".to_string(),
            is_tag_default: true,
            editing: false,
            selected: 1,
            start: Instant::now(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn elapsed_display(&self) -> String {
        let secs = self.start.elapsed().as_secs();
        format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    }

    pub fn handle_key(&mut self, key: KeyCode) -> SessionAction {
        match key {
            // Move cursor
            KeyCode::Down => {
                self.selected = 0;
                SessionAction::None
            }
            KeyCode::Up => {
                self.selected = 1;
                SessionAction::None
            }
            // Tag edition
            KeyCode::Char(c) if self.editing => {
                self.tag.push(c);
                SessionAction::None
            }
            KeyCode::Backspace if self.editing => {
                self.tag.pop();
                SessionAction::None
            }
            KeyCode::Esc | KeyCode::Enter if self.editing => {
                self.editing = false;
                SessionAction::None
            }
            // Exit Session
            KeyCode::Char('q') | KeyCode::Esc => SessionAction::Stop,
            KeyCode::Enter => match self.selected {
                0 => SessionAction::Stop,
                1 if !self.editing => {
                    self.editing = true;
                    // Clear the default placeholder value
                    if self.is_tag_default {
                        self.tag.clear();
                        self.is_tag_default = false;
                    }
                    SessionAction::None
            },
                _ => SessionAction::None,
            },
            _ => SessionAction::None,
        }
    }

    pub fn stop(&mut self) -> (u64, u64) {
        let duration: u64 = self.start.elapsed().as_secs();
        (self.started_at, duration)
    }
}

impl Widget for &Session {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Tic-tac... ".bold());
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<Up/Down>".blue().bold(),
            " Confirm ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let [timer_area, tag_area, stop_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(inner);

        Paragraph::new(self.elapsed_display().bold())
            .centered()
            .render(timer_area, buf);

        let tag_label = if self.editing {
            format!(" < {}_ > ", self.tag)
        } else {
            format!(" < {} > ", self.tag)
        };

        let tag_style = if self.selected == 1 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(vec![
            " [ ".into(),
            tag_label.set_style(tag_style),
            " ] ".into(),
        ]))
        .centered()
        .render(tag_area, buf);

        let stop_style = if self.selected == 0 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(" [ Stop ] ".set_style(stop_style)))
            .centered()
            .render(stop_area, buf);
    }
}
