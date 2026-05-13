use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

mod home;
mod session;
mod stats;

use home::{Home, HomeAction};

enum Screen {
    Home(Home),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Home(Home::default())
    }
}

#[derive(Default)]
pub struct App {
    exit: bool,
    current_screen: Screen,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match &self.current_screen {
            Screen::Home(home) => frame.render_widget(home, frame.area()),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit = true,
                    key => match &mut self.current_screen {
                        Screen::Home(home) => match home.handle_key(key) {
                            HomeAction::StartSession => { /* TODO: transition */ }
                            HomeAction::ViewStats => { /* TODO: transition */ }
                            HomeAction::None => {}
                        },
                    },
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
