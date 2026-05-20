use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug)]
pub struct Session {
    pub label: String,
    is_label_default: bool,
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
            label: "no label".to_string(),
            is_label_default: true,
            editing: false,
            selected: 1,
            start: Instant::now(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            suggestions: suggestions,
            suggestion_state: ListState::default().with_selected(Some(0)),
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
            // Label edition & suggestions navigation reactions
            KeyCode::Char(c) if self.editing => {
                self.label.push(c);
                SessionAction::QueryLabels(self.label.clone())
            }
            KeyCode::Backspace if self.editing => {
                self.label.pop();
                SessionAction::QueryLabels(self.label.clone())
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
                // pick the selected suggestion
                let picked = &self.suggestions[self.suggestion_state.selected().unwrap()];
                self.label = picked.clone();
                self.editing = false;
                SessionAction::None
            }
            // Exit label edition mode
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
                // Label edition mode
                1 if !self.editing => {
                    self.editing = true;
                    // Clear the default placeholder value
                    if self.is_label_default {
                        self.label.clear();
                        self.is_label_default = false;
                    }
                    SessionAction::QueryLabels(self.label.clone())
            },
                _ => SessionAction::None,
            },
            // Exit Session
            KeyCode::Char('q') | KeyCode::Esc => SessionAction::Stop,
            _ => SessionAction::None,
        }
    }
    pub fn stop(&mut self) -> (u64, u64, Option<String>) {
        let duration: u64 = self.start.elapsed().as_secs();
        let label = if self.is_label_default || self.label.trim().is_empty() {
            None 
        } else {
            Some(self.label.clone())
        };
        (self.started_at, duration, label)
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
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);
        
        let [timer_area, label_area, suggestions_area, stop_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(if self.editing && !self.suggestions.is_empty() { self.suggestions.len() as u16 + 2 } else { 0 }),
            Constraint::Fill(1),
        ])
        .areas(inner);

        // Timer
        Paragraph::new(self.elapsed_display().bold())
            .centered()
            .render(timer_area, buf);

        // Label
        let tag_label = if self.editing {
            format!(" < {}_ > ", self.label)
        } else {
            format!(" < {} > ", self.label)
        };
        let label_style = if self.selected == 1 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(vec![
            " [ ".into(),
            tag_label.set_style(label_style),
            " ] ".into(),
        ]))
        .centered()
        .render(label_area, buf);

        // Label suggestions
        if self.editing && !self.suggestions.is_empty() {
            let items: Vec<ListItem> = self.suggestions.iter()
                .map(|l| ListItem::new(l.as_str()))
                .collect();
            let list = List::new(items)
                .highlight_style(Style::new().reversed())
                .block(Block::bordered().title(" Suggestions "));
            StatefulWidget::render(list, suggestions_area, buf, &mut self.suggestion_state);
        }

        let stop_style = if self.selected == 0 { Style::new().reversed() } else { Style::new() };
        Paragraph::new(Line::from(" [ Stop ] ".set_style(stop_style)))
            .centered()
            .render(stop_area, buf);
    }
}
