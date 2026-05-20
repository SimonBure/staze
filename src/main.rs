use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

mod home;
mod session;
mod history;
mod db;

use db::Db;
use home::{Home, HomeAction};
use session::{Session, SessionAction};
use history::{History, HistoryAction, SessionFilter, since_days};

enum Screen {
    Home(Home),
    Session(Session),
    History(History)
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
            Screen::History(stats) => frame.render_widget(stats, frame.area()),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let fail_load_msg = "failed to load history";
        if event::poll(Duration::from_millis(500))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit = true,
                        key => match &mut self.current_screen {
                            Screen::Home(home) => match home.handle_key(key) {
                                HomeAction::StartSession => self.current_screen = Screen::Session(Session::new()),
                                HomeAction::ViewHistory => self.current_screen = {
                                        let month_filter = SessionFilter { since: Some(since_days(30)), tag: None };
                                        let r = self.db.get_sessions(&month_filter).expect(fail_load_msg);
                                        Screen::History(History::new(r))
                                },
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
                            Screen::History(hist) => match hist.handle_key(key) {
                                HistoryAction::Stop => {
                                    self.current_screen = Screen::Home(Home::default());
                                },
                                HistoryAction::Query(filter) => {
                                    let r = self.db.get_sessions(&filter).expect(fail_load_msg);
                                    hist.sessions = r;
                                }
                                HistoryAction::None => {},
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
