use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct Home {
    selected: u8,
    can_undo: bool,
}

pub enum HomeAction {
    None,
    StartSession,
    ViewHistory,
    ViewTags,
    UndoLastSession,
    ResumeLastSession,
}

impl Home {
    pub fn new(can_undo: bool) -> Self {
        Self { selected: 0, can_undo }
    }
}

impl Home {
    pub fn handle_key(&mut self, key: KeyCode) -> HomeAction {
        match key {
            KeyCode::Left => {
                self.selected = self.selected.saturating_sub(1);
                HomeAction::None
            }
            KeyCode::Right => {
                self.selected = (self.selected + 1).min(2);
                HomeAction::None
            }
            KeyCode::Enter => match self.selected {
                0 => HomeAction::StartSession,
                1 => HomeAction::ViewHistory,
                _ => HomeAction::ViewTags,
            },
            KeyCode::Char('u') | KeyCode::Char('U') if self.can_undo => HomeAction::UndoLastSession,
            KeyCode::Char('r') | KeyCode::Char('R') if self.can_undo => HomeAction::ResumeLastSession,
            _ => HomeAction::None,
        }
    }
}

impl Widget for &mut Home {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Let's get to work! ".bold());
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<Left/Right>".blue().bold(),
            " Select ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let start_style = if self.selected == 0 { Style::new().reversed() } else { Style::new() };
        let stats_style = if self.selected == 1 { Style::new().reversed() } else { Style::new() };
        let tags_style  = if self.selected == 2 { Style::new().reversed() } else { Style::new() };

        let buttons = Line::from(vec![
            " [ Start Session ] ".set_style(start_style),
            "   ".into(),
            " [ View History ] ".set_style(stats_style),
            "   ".into(),
            " [ Manage Tags ] ".set_style(tags_style),
        ]);

        let mut lines = vec![buttons];
        if self.can_undo {
            lines.push(Line::from("Press U to undo  ·  Press R to resume").centered().dark_gray());
        }
        Paragraph::new(lines)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
