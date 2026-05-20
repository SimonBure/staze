use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::db::SessionRecord;


fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{}h {:02}m", h, m)
}

pub fn since_days(days: u64) -> i64 {
    let cutoff = SystemTime::now() - Duration::from_secs(days * 86400);
    cutoff.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub struct SessionFilter {
    pub since: Option<i64>,
    pub tag: Option<String>,
}

pub struct History {
    pub sessions: Vec<SessionRecord>,
    selected: u8,
}

pub enum HistoryAction {
    None,
    Stop,
    Query(SessionFilter)
}

impl History {
    pub fn new(sessions: Vec<SessionRecord>) -> Self {
        Self {
            selected: 1,  // display month data by default
            sessions: sessions
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> HistoryAction {
        match key {
            KeyCode::Left => {
                self.selected = (self.selected - 1).saturating_sub(0);
                HistoryAction::None
            },
            KeyCode::Right => {
                self.selected = (self.selected + 1).min(2);
                HistoryAction::None
            }
            KeyCode::Enter => {
                match self.selected {
                    0 => HistoryAction::Query(SessionFilter { since: Some(since_days(7)), tag: None}),
                    1 => HistoryAction::Query(SessionFilter { since: Some(since_days(30)), tag: None}),
                    2 => HistoryAction::Query(SessionFilter { since: Some(since_days(365)), tag: None}),
                    _ => HistoryAction::None,
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => HistoryAction::Stop,
            _ => HistoryAction::None,
        }
    }

    fn get_sessions_nb(&self) -> usize {
        return self.sessions.len()
    }
    fn get_total_worked(&self) -> i64 {
        self.sessions.iter().map(|s| s.duration_sec).sum()
    }
}

impl Widget for &History {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Have you worked well? ".bold());
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

        let style = |i| if self.selected == i {Style::new().reversed()} else {Style::new()};
        let week_style = style(0);
        let month_style = style(1);
        let year_style = style(2);
        
        let total_duration = self.get_total_worked();

        let content = vec![
            Line::from(vec![
            " [ Week ] ".set_style(week_style),
            "   ".into(),
            " [ Month ] ".set_style(month_style),
            "   ".into(),
            " [ Year ] ".set_style(year_style),
            ]),
            Line::from(vec!["Total Worked:".into(), format_duration(total_duration).bold()]),
            ];
        
        Paragraph::new(content)
            .centered()
            .block(block)
            .render(area, buf);
    }
    
}