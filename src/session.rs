use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, ListState, Paragraph, StatefulWidget, Widget},
};
use tui_big_text::{BigText, PixelSize};

use crate::label_input::{InputEvent, LabelInput};
use crate::starfield::{Starfield, STAR_COUNT};

#[derive(Debug)]
pub struct Session {
    pub label: Option<String>,
    selected: u8,
    start: Instant,
    started_at: u64,
    input: LabelInput,
    stars: Starfield,
}

pub enum SessionAction {
    None,
    Stop,
    QueryLabels(String),
}

impl Session {
    pub fn new() -> Self {
        Self {
            label: None,
            selected: 1,
            start: Instant::now(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            input: LabelInput::default(),
            stars: Starfield::new(STAR_COUNT),
        }
    }

    pub fn resume(started_at: u64, elapsed_secs: u64, label: Option<String>) -> Self {
        Self {
            label,
            selected: 1,
            start: Instant::now() - Duration::from_secs(elapsed_secs),
            started_at,
            input: LabelInput::default(),
            stars: Starfield::new(STAR_COUNT),
        }
    }

    pub fn is_typing(&self) -> bool {
        self.input.is_active()
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.input.update_suggestions(suggestions);
    }

    fn elapsed_display(&self) -> String {
        let secs = self.start.elapsed().as_secs();
        format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    }

    pub fn handle_key(&mut self, key: KeyCode) -> SessionAction {
        // Label edition & suggestions navigation
        if self.input.is_active() {
            return match self.input.handle_key(key) {
                InputEvent::Query(prefix) => SessionAction::QueryLabels(prefix),
                InputEvent::Submit(label) => {
                    self.label = label;
                    SessionAction::None
                }
                InputEvent::Cancel | InputEvent::None => SessionAction::None,
            };
        }
        match key {
            // Cursor navigation
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = 0;
                SessionAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = 1;
                SessionAction::None
            }
            // One-shot label edit from anywhere
            KeyCode::Char('/') => {
                self.selected = 1;
                SessionAction::QueryLabels(self.input.open(self.label.as_deref()))
            }
            KeyCode::Enter => match self.selected {
                0 => SessionAction::Stop,
                _ => SessionAction::QueryLabels(self.input.open(self.label.as_deref())),
            },
            KeyCode::Char('q') | KeyCode::Esc => SessionAction::Stop,
            _ => SessionAction::None,
        }
    }

    pub fn stop(&mut self) -> (u64, u64, Option<String>) {
        let duration: u64 = self.start.elapsed().as_secs();
        (self.started_at, duration, self.label.clone())
    }
}

impl StatefulWidget for &mut Session {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut ListState) {
        let title = Line::from(" Working hard... ".bold());
        let instructions = if self.input.is_active() {
            LabelInput::instructions()
        } else {
            Line::from(vec![
                " Navigate ".into(),
                "<Up/Down> ; <k/j>".blue().bold(),
                " Confirm ".into(),
                "<Enter>".blue().bold(),
                " Label ".into(),
                "</>".blue().bold(),
                " Stop ".into(),
                "<Esc> ".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ])
        };

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        // Layout definition
        let [_top, running_area, _gap, timer_display_area, _mid1, label_area, hint_area, suggestions_area, _mid2, stop_area, _bottom] = Layout::vertical([
            Constraint::Fill(1),  // top filler
            Constraint::Length(1),  // "session is running" space 
            Constraint::Length(1),  // whitespace between timer and runner
            Constraint::Length(4),  // timer area
            Constraint::Length(1),   // whitespace between timer and label
            Constraint::Length(1),  // label area
            // Suggestion & hint areas
            Constraint::Length(if self.selected == 1 && !self.input.is_active() { 1 } else { 0 }),
            Constraint::Length(self.input.dropdown_height()),
            Constraint::Fill(1),
            Constraint::Length(1),  // Stop area
            Constraint::Length(1), // fixed whitespace after Stop
        ]).areas(inner);
        
        // Timer
        Paragraph::new(Line::from("● session in progress".green()))
            .centered()
            .render(running_area, buf);
        BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .lines(vec![Line::from(self.elapsed_display().green().bold())])
            .centered()
            .build()
            .render(timer_display_area, buf);

        // Label
        let tag_label = self.input.display(self.label.as_deref(), " no label ");
        let label_style = if self.selected == 1 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(vec![
            " [ ".into(),
            tag_label.set_style(label_style),
            " ] ".into(),
        ]))
        .centered()
        .render(label_area, buf);

        if self.selected == 1 && !self.input.is_active() {
            Paragraph::new(Line::from("Press Enter to label this session".dark_gray()))
                .centered()
                .render(hint_area, buf);
        }

        self.input.render_dropdown(suggestions_area, buf, " Suggestions ");

        // Stop
        let stop_style = if self.selected == 0 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(" [ Stop ] ".set_style(stop_style)))
            .centered()
            .render(stop_area, buf);

        // Last, so stars only fill the cells left blank
        self.stars.render(inner, buf);
    }
}
