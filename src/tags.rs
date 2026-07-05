use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, ListState, Paragraph, StatefulWidget, Widget},
};

pub struct Tags {
    tags: Vec<(String, usize)>,
    selected: usize,
    editing: bool,
    edit_buf: String,
}

pub enum TagsAction {
    None,
    Stop,
    Delete(String),
    Rename { old: String, new: String },
}

impl Tags {
    pub fn new(tags: Vec<(String, usize)>) -> Self {
        Self { tags, selected: 0, editing: false, edit_buf: String::new() }
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn update(&mut self, tags: Vec<(String, usize)>) {
        self.tags = tags;
        if self.selected >= self.tags.len() && !self.tags.is_empty() {
            self.selected = self.tags.len() - 1;
        }
        self.editing = false;
        self.edit_buf.clear();
    }

    pub fn handle_key(&mut self, key: KeyCode) -> TagsAction {
        if self.editing {
            match key {
                KeyCode::Char(c) => {
                    self.edit_buf.push(c);
                    TagsAction::None
                }
                KeyCode::Backspace => {
                    self.edit_buf.pop();
                    TagsAction::None
                }
                KeyCode::Enter => {
                    let new = self.edit_buf.trim().to_string();
                    if new.is_empty() {
                        self.editing = false;
                        self.edit_buf.clear();
                        return TagsAction::None;
                    }
                    let old = self.tags[self.selected].0.clone();
                    self.editing = false;
                    self.edit_buf.clear();
                    if new == old {
                        TagsAction::None
                    } else {
                        TagsAction::Rename { old, new }
                    }
                }
                KeyCode::Esc => {
                    self.editing = false;
                    self.edit_buf.clear();
                    TagsAction::None
                }
                _ => TagsAction::None,
            }
        } else {
            match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 { self.selected -= 1; }
                    TagsAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected + 1 < self.tags.len() { self.selected += 1; }
                    TagsAction::None
                }
                KeyCode::Enter => {
                    if !self.tags.is_empty() {
                        self.edit_buf = self.tags[self.selected].0.clone();
                        self.editing = true;
                    }
                    TagsAction::None
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if !self.tags.is_empty() {
                        TagsAction::Delete(self.tags[self.selected].0.clone())
                    } else {
                        TagsAction::None
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => TagsAction::Stop,
                _ => TagsAction::None,
            }
        }
    }
}

impl StatefulWidget for &mut Tags {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut ListState) {
        let hint = if self.editing {
            Line::from(vec![
                " Confirm ".into(),
                "<Enter>".blue().bold(),
                "  Cancel ".into(),
                "<Esc> ".blue().bold(),
            ])
        } else {
            Line::from(vec![
                " Navigate ".into(),
                "<↑↓> ; <k/j>".blue().bold(),
                "  Rename ".into(),
                "<Enter>".blue().bold(),
                "  Delete ".into(),
                "<D>".blue().bold(),
                "  Back ".into(),
                "<Esc> ".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ])
        };

        let block = Block::bordered()
            .title(Line::from(" Manage Tags ".bold()).centered())
            .title_bottom(hint.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.tags.is_empty() {
            Paragraph::new("No tags yet.")
                .centered()
                .render(inner, buf);
            return;
        }

        let rows: Vec<Constraint> = self.tags.iter().map(|_| Constraint::Length(1)).collect();
        let cells = Layout::vertical(rows).split(inner);

        for (i, ((label, count), cell)) in self.tags.iter().zip(cells.iter()).enumerate() {
            let is_selected = i == self.selected;
            let text = if is_selected && self.editing {
                format!("  {}_ ({} sessions)", self.edit_buf, count)
            } else {
                format!("  {} ({} sessions)", label, count)
            };
            let style = if is_selected { Style::new().reversed() } else { Style::new() };
            Paragraph::new(text).set_style(style).render(*cell, buf);
        }
    }
}
