use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub struct Stats {
    selected: u8,
    view: StatsView,
}

enum StatsView {
    Home,
    Graphs,
}

pub enum StatsAction {
    None,
    Stop,
    GetAllDataLabel,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            selected: 0,
            view: StatsView::Home,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> StatsAction {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => StatsAction::Stop,
            _ => StatsAction::None,
        }
    }
}

impl Widget for &Stats {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Different rendering logic based on the action:
        // Default view => #hours worked the past month (/week/year -> selectable) 
        // Graphs view => time line with working hours the past week/month/year
        //
        let title = Line::from(" Have you worked well? ".bold());
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<Left/Right>".blue().bold(),
            " Select ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        
        match self.view {
            StatsView::Home => {
                let graph_style = if self.selected == 1 {
                    Style::new().reversed()
                } else {
                    Style::new()
                };
                let content = Line::from(vec![
                    " [  Graphs ] ".set_style(graph_style),
                    "   ".into(),
                    " [ View Stats ] ".set_style(stats_style),
                ]);
            },
            StatsView::Graphs => {
                
            }
        }
        

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

        Paragraph::new(content)
            .centered()
            .block(block)
            .render(area, buf);
    }
}