use std::io;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

mod config;
mod home;
mod session;
mod history;
mod db;

use db::{Db, SessionFilter};

struct LastSession {
    id: i64,
    started_at: u64,
    duration_sec: u64,
    label: Option<String>,
}
use home::{Home, HomeAction};
use session::{Session, SessionAction};
use history::{History, HistoryAction};

fn since_days(days: u64) -> i64 {
    let cutoff = SystemTime::now() - Duration::from_secs(days * 86400);
    cutoff.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

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
    last_session: Option<LastSession>,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match &mut self.current_screen {
            Screen::Home(home) => frame.render_widget(home, frame.area()),
            Screen::Session(session) => frame.render_stateful_widget(session, frame.area(), &mut ListState::default()),
            Screen::History(history) => frame.render_stateful_widget(history, frame.area(), &mut ListState::default()),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let fail_load_history = "failed to load history";
        let fail_load_label = "failed to fetch labels";
        if event::poll(Duration::from_millis(500))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') => self.exit = true,
                        key => match &mut self.current_screen {
                            Screen::Home(home) => match home.handle_key(key) {
                                HomeAction::StartSession => {
                                    self.last_session = None;
                                    let suggestions = self.db.get_labels("").expect(fail_load_label);
                                    self.current_screen = Screen::Session(Session::new(suggestions));
                                },
                                HomeAction::UndoLastSession => {
                                    if let Some(ls) = self.last_session.take() {
                                        self.db.delete_session(ls.id).expect("failed to undo session");
                                    }
                                    self.current_screen = Screen::Home(Home::new(false));
                                },
                                HomeAction::ResumeLastSession => {
                                    if let Some(ls) = self.last_session.take() {
                                        self.db.delete_session(ls.id).expect("failed to delete session for resume");
                                        let suggestions = self.db.get_labels("").expect(fail_load_label);
                                        self.current_screen = Screen::Session(
                                            Session::resume(ls.started_at, ls.duration_sec, ls.label, suggestions)
                                        );
                                    }
                                },
                                HomeAction::ViewHistory => {
                                    let month_filter = SessionFilter { since: Some(since_days(30)), tag: None };
                                    let r = self.db.get_sessions(&month_filter).expect(fail_load_history);
                                    let suggestions = self.db.get_labels("").expect(fail_load_label);
                                    let mut h = History::new(r);
                                    h.update_suggestions(suggestions);
                                    self.current_screen = Screen::History(h);
                                }
                                HomeAction::None => {}
                            },
                            Screen::Session(session) => match session.handle_key(key) {
                                SessionAction::QueryLabels(prefix) => {
                                    let suggestions = self.db.get_labels(&prefix).expect(fail_load_label);
                                    session.update_suggestions(suggestions);
                                },
                                SessionAction::Stop => {
                                    let (started_at, duration_sec, label) = session.stop();
                                    let id = self.db.save_session(started_at, duration_sec, label.clone()).expect("failed to save session");
                                    self.last_session = Some(LastSession { id, started_at, duration_sec, label });
                                    self.current_screen = Screen::Home(Home::new(true));
                                }
                                SessionAction::None => {}
                            },
                            Screen::History(hist) => match hist.handle_key(key) {
                                HistoryAction::Stop => self.current_screen = Screen::Home(Home::default()),
                                HistoryAction::Query(selected, label) => {
                                    let days = match selected { 0 => 7, 1 => 30, _ => 365 };
                                    let filter = SessionFilter { since: Some(since_days(days)), tag: label };
                                    let r = self.db.get_sessions(&filter).expect(fail_load_history);
                                    hist.update(r);
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
    let cfg = config::Config::load();
    let db_path = cfg.resolved_db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = Db::open(&db_path).expect("failed to open the database");
    ratatui::run(|terminal| App {
        exit: false,
        current_screen: Screen::default(),
        db,
        last_session: None,
    }
    .run(terminal))
}
