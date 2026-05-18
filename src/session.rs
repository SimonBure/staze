use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct Session {
    pub tag: String,
    selected: u8,
    start: Instant,
    started_at: u64
}

pub enum SessionAction {
    None,
    Stop,
}

impl Session {
    pub fn new() -> Self {
        Self{
            tag: "wonderful-thinking-session".to_string(),
            selected: 1,  // default on the <tag> space for quick edition 
            start: Instant::now(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    pub fn handle_key(&mut self, key: KeyCode) -> SessionAction {
        match key {
            KeyCode::Down => {
                self.selected = 0;
                SessionAction::None
            }
            KeyCode::Up => {
                self.selected = 1;
                SessionAction::None
            }
            KeyCode::Char('q') | KeyCode::Esc => SessionAction::Stop,
            KeyCode::Enter => match self.selected {
                0 => SessionAction::Stop,
                // 1 => { /* Write the tag's name or select among existing tags*/ }  
                _ => SessionAction::None,
            },
            _ => SessionAction::None,
        }
    }

    pub fn stop(&mut self) -> (u64, u64) {
        // Compute the duration
        let duration: u64 = self.started.elapsed().as_secs();
        return (self.started_at, duration)
    }
}

impl Widget for &Session {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Dynamic timer rendering
    }
}