use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, List, ListItem, ListState, StatefulWidget, Widget},
};

const MAX_VISIBLE: usize = 5;

/// Label text input with DB-backed autocomplete, shared by the session, history and tags screens.
/// The owning screen keeps the committed value; this only holds the buffer while editing.
#[derive(Debug, Default)]
pub struct LabelInput {
    text: String,
    active: bool,
    suggestions: Vec<String>,
    state: ListState,
}

pub enum InputEvent {
    None,
    /// Text changed: App should refresh suggestions for this prefix.
    Query(String),
    /// Enter: the highlighted suggestion if any, else the typed text (`None` if empty).
    Submit(Option<String>),
    Cancel,
}

impl LabelInput {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Start editing from `initial`; returns the prefix to query suggestions with.
    pub fn open(&mut self, initial: Option<&str>) -> String {
        self.text = initial.unwrap_or("").to_string();
        self.active = true;
        self.state.select(None);
        self.text.clone()
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        self.state.select(None);
    }

    pub fn handle_key(&mut self, key: KeyCode) -> InputEvent {
        match key {
            KeyCode::Char(c) => {
                self.text.push(c);
                InputEvent::Query(self.text.clone())
            }
            KeyCode::Backspace => {
                self.text.pop();
                InputEvent::Query(self.text.clone())
            }
            KeyCode::Down => {
                self.state.select_next();
                InputEvent::None
            }
            KeyCode::Up => {
                self.state.select_previous();
                InputEvent::None
            }
            KeyCode::Enter => {
                if let Some(picked) = self.state.selected().and_then(|i| self.suggestions.get(i)) {
                    self.text = picked.clone();
                }
                self.close();
                let text = self.text.trim();
                InputEvent::Submit((!text.is_empty()).then(|| text.to_string()))
            }
            KeyCode::Esc => {
                self.close();
                InputEvent::Cancel
            }
            _ => InputEvent::None,
        }
    }

    fn close(&mut self) {
        self.active = false;
        self.state.select(None);
    }

    /// ` < text_ > ` while editing, ` < value > ` when set, `placeholder` otherwise.
    pub fn display(&self, value: Option<&str>, placeholder: &str) -> String {
        match value {
            _ if self.active => format!(" {}_ ", self.text),
            Some(v) => format!(" {} ", v),
            None => placeholder.to_string(),
        }
    }

    pub fn instructions() -> Line<'static> {
        Line::from(vec![
            " Pick ".into(),
            "<Up/Down>".blue().bold(),
            " Confirm ".into(),
            "<Enter>".blue().bold(),
            " Cancel ".into(),
            "<Esc> ".blue().bold(),
        ])
    }

    /// Rows the dropdown needs (0 when hidden).
    pub fn dropdown_height(&self) -> u16 {
        if self.active && !self.suggestions.is_empty() {
            self.suggestions.len().min(MAX_VISIBLE) as u16 + 2
        } else { 0 }
    }

    /// Draws the dropdown centered at the top of `area`, over whatever is there.
    pub fn render_dropdown(&mut self, area: Rect, buf: &mut Buffer, title: &str) {
        let height = self.dropdown_height().min(area.height);
        if height == 0 { return; }
        let width = self.suggestions.iter()
            .map(|l| l.chars().count() as u16)
            .max()
            .unwrap_or(0)
            .saturating_add(4)
            .clamp(20.min(area.width), area.width);
        let rect = Rect { x: area.x + (area.width - width) / 2, y: area.y, width, height };

        let items: Vec<ListItem> = self.suggestions.iter()
            .map(|l| ListItem::new(l.as_str()))
            .collect();
        let list = List::new(items)
            .highlight_style(Style::new().reversed())
            .block(Block::bordered().title(title.to_string()));
        Clear.render(rect, buf);
        StatefulWidget::render(list, rect, buf, &mut self.state);
    }
}
