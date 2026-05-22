use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use tui_big_text::{BigText, PixelSize};

#[derive(Debug)]
pub struct Session {
    pub label: Option<String>,
    editing: bool,
    selected: u8,
    start: Instant,
    started_at: u64,
    suggestions: Vec<String>,
    suggestion_state: ListState,
}

pub enum SessionAction {
    None,
    Stop,
    QueryLabels(String),
}

impl Session {
    pub fn new(suggestions: Vec<String>) -> Self {
        Self {
            label: None,
            editing: false,
            selected: 1,
            start: Instant::now(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            suggestions,
            suggestion_state: ListState::default(),
        }
    }

    pub fn resume(started_at: u64, elapsed_secs: u64, label: Option<String>, suggestions: Vec<String>) -> Self {
        Self {
            label,
            editing: false,
            selected: 1,
            start: Instant::now() - Duration::from_secs(elapsed_secs),
            started_at,
            suggestions,
            suggestion_state: ListState::default(),
        }
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        self.suggestion_state.select(None);
    }

    fn elapsed_display(&self) -> String {
        let secs = self.start.elapsed().as_secs();
        format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    }

    pub fn handle_key(&mut self, key: KeyCode) -> SessionAction {
        match key {
            // Label edition & suggestions navigation
            KeyCode::Char(c) if self.editing => {
                self.label.get_or_insert_with(String::new).push(c);
                SessionAction::QueryLabels(self.label.as_deref().unwrap_or("").to_string())
            }
            KeyCode::Backspace if self.editing => {
                if let Some(ref mut l) = self.label {
                    l.pop();
                    if l.is_empty() { self.label = None; }
                }
                SessionAction::QueryLabels(self.label.as_deref().unwrap_or("").to_string())
            }
            KeyCode::Down if self.editing => {
                self.suggestion_state.select_next();
                SessionAction::None
            }
            KeyCode::Up if self.editing => {
                self.suggestion_state.select_previous();
                SessionAction::None
            }
            KeyCode::Enter if self.editing && self.suggestion_state.selected().is_some() => {
                if let Some(i) = self.suggestion_state.selected()
                    && let Some(picked) = self.suggestions.get(i) {
                        self.label = Some(picked.clone());
                    }
                self.editing = false;
                self.suggestion_state.select(None);
                SessionAction::None
            }
            KeyCode::Esc | KeyCode::Enter if self.editing => {
                self.editing = false;
                SessionAction::None
            }
            // Cursor navigation
            KeyCode::Down => {
                self.selected = 0;
                SessionAction::None
            }
            KeyCode::Up => {
                self.selected = 1;
                SessionAction::None
            }
            KeyCode::Enter => match self.selected {
                0 => SessionAction::Stop,
                1 if !self.editing => {
                    self.editing = true;
                    SessionAction::QueryLabels(self.label.as_deref().unwrap_or("").to_string())
                }
                _ => SessionAction::None,
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
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<Up/Down>".blue().bold(),
            " Confirm ".into(),
            "<Enter>".blue().bold(),
            " Stop ".into(),
            "<Esc> ".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

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
            Constraint::Length(if self.selected == 1 && !self.editing { 1 } else { 0 }),
            Constraint::Length(if self.editing && !self.suggestions.is_empty() {
                self.suggestions.len() as u16 + 2
            } else { 0 }),
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
        let tag_label = match &self.label {
            Some(l) if self.editing => format!(" < {}_ > ", l),
            Some(l)              => format!(" < {} > ", l),
            None if self.editing => " < _ > ".to_string(),
            None                 => " [ no label ] ".to_string(),
        };
        let label_style = if self.selected == 1 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(vec![
            " [ ".into(),
            tag_label.set_style(label_style),
            " ] ".into(),
        ]))
        .centered()
        .render(label_area, buf);

        if self.selected == 1 && !self.editing {
            Paragraph::new(Line::from("Press Enter to label this session".dark_gray()))
                .centered()
                .render(hint_area, buf);
        }

        if self.editing && !self.suggestions.is_empty() {
            let items: Vec<ListItem> = self.suggestions.iter()
                .map(|l| ListItem::new(l.as_str()))
                .collect();
            let list = List::new(items)
                .highlight_style(Style::new().reversed())
                .block(Block::bordered().title(" Suggestions "));
            StatefulWidget::render(list, suggestions_area, buf, &mut self.suggestion_state);
        }

        // Stop
        let stop_style = if self.selected == 0 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(" [ Stop ] ".set_style(stop_style)))
            .centered()
            .render(stop_area, buf);
    }
}
