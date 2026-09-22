use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::label_input::{InputEvent, LabelInput};

#[derive(PartialEq)]
enum Mode {
    Browse,
    Search,
    Rename,
}

pub struct Tags {
    tags: Vec<(String, usize)>,
    selected: usize,
    mode: Mode,
    input: LabelInput,
}

pub enum TagsAction {
    None,
    Stop,
    QueryLabels(String),
    Delete(String),
    Rename { old: String, new: String },
}

impl Tags {
    pub fn new(tags: Vec<(String, usize)>) -> Self {
        Self { tags, selected: 0, mode: Mode::Browse, input: LabelInput::default() }
    }

    pub fn is_typing(&self) -> bool {
        self.input.is_active()
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.input.update_suggestions(suggestions);
    }

    pub fn update(&mut self, tags: Vec<(String, usize)>) {
        self.tags = tags;
        if self.selected >= self.tags.len() && !self.tags.is_empty() {
            self.selected = self.tags.len() - 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> TagsAction {
        if self.input.is_active() {
            return match self.input.handle_key(key) {
                InputEvent::Query(prefix) => TagsAction::QueryLabels(prefix),
                InputEvent::Submit(value) => {
                    let mode = std::mem::replace(&mut self.mode, Mode::Browse);
                    let Some(value) = value else { return TagsAction::None };
                    match mode {
                        // Jump to the picked tag
                        Mode::Search => {
                            if let Some(i) = self.tags.iter().position(|(l, _)| *l == value) {
                                self.selected = i;
                            }
                            TagsAction::None
                        }
                        // Renaming onto an existing tag merges the two
                        Mode::Rename => {
                            let old = self.tags[self.selected].0.clone();
                            if value == old { TagsAction::None } else { TagsAction::Rename { old, new: value } }
                        }
                        Mode::Browse => TagsAction::None,
                    }
                }
                InputEvent::Cancel => {
                    self.mode = Mode::Browse;
                    TagsAction::None
                }
                InputEvent::None => TagsAction::None,
            };
        }
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 { self.selected -= 1; }
                TagsAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.tags.len() { self.selected += 1; }
                TagsAction::None
            }
            KeyCode::Char('/') if !self.tags.is_empty() => {
                self.mode = Mode::Search;
                TagsAction::QueryLabels(self.input.open(None))
            }
            KeyCode::Enter if !self.tags.is_empty() => {
                self.mode = Mode::Rename;
                TagsAction::QueryLabels(self.input.open(Some(&self.tags[self.selected].0)))
            }
            KeyCode::Char('d') | KeyCode::Char('D') if !self.tags.is_empty() => {
                TagsAction::Delete(self.tags[self.selected].0.clone())
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => TagsAction::Stop,
            _ => TagsAction::None,
        }
    }
}

impl StatefulWidget for &mut Tags {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut ListState) {
        let hint = if self.input.is_active() {
            LabelInput::instructions()
        } else {
            Line::from(vec![
                " Navigate ".into(),
                "<↑↓> ; <k/j>".blue().bold(),
                "  Search ".into(),
                "</>".blue().bold(),
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

        let [search_area, list_area] = Layout::vertical([
            Constraint::Length(if self.mode == Mode::Search { 1 } else { 0 }),
            Constraint::Fill(1),
        ]).areas(inner);

        if self.mode == Mode::Search {
            Paragraph::new(format!(" / {}_", self.input.text()))
                .render(search_area, buf);
        }

        // Scroll so the selected row stays visible
        let visible = list_area.height as usize;
        let offset = (self.selected + 1).saturating_sub(visible);

        for (row, (i, (label, count))) in self.tags.iter().enumerate().skip(offset).take(visible).enumerate() {
            let cell = Rect { y: list_area.y + row as u16, height: 1, ..list_area };
            let is_selected = i == self.selected;
            let text = if is_selected && self.mode == Mode::Rename {
                format!("  {}_ ({} sessions)", self.input.text(), count)
            } else {
                format!("  {} ({} sessions)", label, count)
            };
            let style = if is_selected { Style::new().reversed() } else { Style::new() };
            Paragraph::new(text).set_style(style).render(cell, buf);
        }

        // Dropdown under the search bar, or under the row being renamed
        let dropdown_area = match self.mode {
            Mode::Rename => {
                let y = list_area.y + (self.selected - offset) as u16 + 1;
                Rect { y, height: list_area.bottom().saturating_sub(y), ..list_area }
            }
            _ => list_area,
        };
        self.input.render_dropdown(dropdown_area, buf, " Suggestions ");
    }
}
