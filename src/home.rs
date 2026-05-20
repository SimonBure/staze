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
}

pub enum HomeAction {
    None,
    StartSession,
    ViewHistory,
}

impl Home {
    pub fn handle_key(&mut self, key: KeyCode) -> HomeAction {
        match key {
            KeyCode::Left => {
                self.selected = 0;
                HomeAction::None
            }
            KeyCode::Right => {
                self.selected = 1;
                HomeAction::None
            }
            KeyCode::Enter => match self.selected {
                0 => HomeAction::StartSession,
                _ => HomeAction::ViewHistory,
            },
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

        let start_style = if self.selected == 0 {
            Style::new().reversed()
        } else {
            Style::new()
        };
        let stats_style = if self.selected == 1 {
            Style::new().reversed()
        } else {
            Style::new()
        };

        let content = Line::from(vec![
            " [ Start Session ] ".set_style(start_style),
            "   ".into(),
            " [ View History ] ".set_style(stats_style),
        ]);

        Paragraph::new(content)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
