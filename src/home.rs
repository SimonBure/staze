use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::appearance;
use crate::galaxy::{Galaxy, FIELD_STAR_COUNT};
use crate::starfield::Starfield;

#[derive(Debug)]
pub struct Home {
    selected: u8,
    can_undo: bool,
    galaxy: Galaxy,
    stars: Starfield,
}

pub enum HomeAction {
    None,
    StartSession,
    ViewHistory,
    ViewTags,
    ViewSettings,
    UndoLastSession,
    ResumeLastSession,
}

impl Home {
    pub fn new(can_undo: bool) -> Self {
        Self {
            selected: 0,
            can_undo,
            galaxy: Galaxy,
            stars: Starfield::new(FIELD_STAR_COUNT).without_meteors(),
        }
    }
}

impl Default for Home {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Home {
    pub fn handle_key(&mut self, key: KeyCode) -> HomeAction {
        match key {
            KeyCode::Left | KeyCode::Char('h') => {
                self.selected = self.selected.saturating_sub(1);
                HomeAction::None
            }
            KeyCode::Right | KeyCode::Char('l')=> {
                self.selected = (self.selected + 1).min(3);
                HomeAction::None
            }
            KeyCode::Enter => match self.selected {
                0 => HomeAction::StartSession,
                1 => HomeAction::ViewHistory,
                2 => HomeAction::ViewTags,
                _ => HomeAction::ViewSettings,
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
            "<Left/Right> ; <h/l>".blue().bold(),
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
        let settings_style = if self.selected == 3 { Style::new().reversed() } else { Style::new() };

        let buttons = Line::from(vec![
            " [ Start Session ] ".set_style(start_style),
            "   ".into(),
            " [ View History ] ".set_style(stats_style),
            "   ".into(),
            " [ Manage Tags ] ".set_style(tags_style),
            "   ".into(),
            " [ Settings ] ".set_style(settings_style),
        ]);

        let inner = block.inner(area);
        let mut lines = vec![buttons];
        if self.can_undo {
            lines.push(Line::from("Press U to undo  ·  Press R to resume").centered().dark_gray());
        }
        Paragraph::new(lines)
            .centered()
            .block(block)
            .render(area, buf);

        // Last, so the sky only fills the cells left blank
        if appearance::galaxy() {
            self.galaxy.render(inner, buf);
            self.stars.render(inner, buf);
        }
    }
}
