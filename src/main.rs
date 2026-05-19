use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

mod home;
mod session;
mod stats;
mod db;

use db::Db;
use home::{Home, HomeAction};
use session::{Session, SessionAction};
use stats::{Stats, StatsAction};

enum Screen {
    Home(Home),
    Session(Session),
    Stats(Stats)
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Home(Home::default())
    }
}

pub struct App {
    exit: bool,
    current_screen: Screen,
    db: Db,
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
            Screen::Session(session) => frame.render_widget(session, frame.area()),
            Screen::Stats(stats) => frame.render_widget(stats, frame.area()),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(500))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit = true,
                        key => match &mut self.current_screen {
                            Screen::Home(home) => match home.handle_key(key) {
                                HomeAction::StartSession => self.current_screen = Screen::Session(Session::new()),
                                HomeAction::ViewStats => { /* TODO: new view */ }
                                HomeAction::None => {}
                            },
                            Screen::Session(session) => match session.handle_key(key) {
                                SessionAction::Stop => {
                                    let (start, duration, label) = session.stop();
                                    self.db.save_session(start, duration, label).expect("failed to save session");
                                    self.current_screen = Screen::Home(Home::default());
                                }
                                SessionAction::None => {}
                            },
                            Screen::Stats(stats) => match stats.handle_key(key) {
                                
                                StatsAction::None => {}
                            }
                        },
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let db = Db::open().expect("failed to open the database");
    ratatui::run(|terminal| App {
        exit: false,
        current_screen: Screen::default(),
        db}
    .run(terminal))
}
