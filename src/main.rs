use std::io;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

mod config;
mod home;
mod session;
mod history;
mod db;
mod tags;
mod label_input;
mod starfield;
mod galaxy;
mod export;


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
use tags::{Tags, TagsAction};

use crate::config::Config;

fn since_days(days: u64) -> i64 {
    let cutoff = SystemTime::now() - Duration::from_secs(days * 86400);
    cutoff.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

enum Screen {
    Home(Home),
    Session(Session),
    History(History),
    Tags(Tags),
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
        let frame_budget = Duration::from_millis(100);  // 10 fps rendering
        let mut next_frame = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            // The sky only animates on Home/Session; on other screens block until
            // the next key so the app sits at ~0% CPU when idle.
            let animated = matches!(self.current_screen, Screen::Home(_) | Screen::Session(_));
            let timeout = if animated {
                next_frame.saturating_duration_since(Instant::now())
            } else {
                Duration::from_secs(3600)
            };
            self.handle_events(timeout)?;
            let now = Instant::now();
            if now >= next_frame {
                next_frame += frame_budget;
                if next_frame < now { next_frame = now + frame_budget; }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match &mut self.current_screen {
            Screen::Home(home) => frame.render_widget(home, area),
            Screen::Session(session) => frame.render_stateful_widget(session, area, &mut ListState::default()),
            Screen::History(history) => frame.render_stateful_widget(history, area, &mut ListState::default()),
            Screen::Tags(tags) => frame.render_stateful_widget(tags, area, &mut ListState::default()),
        }
    }

    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        let fail_load_history = "failed to load history";
        let fail_load_label = "failed to fetch labels";
        
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let typing = match &self.current_screen {
                        Screen::Home(_) => false,
                        Screen::Session(s) => s.is_typing(),
                        Screen::History(h) => h.is_typing(),
                        Screen::Tags(t) => t.is_typing(),
                    };
                    match key_event.code {
                        KeyCode::Char('q') if !typing => self.exit = true,
                        key => match &mut self.current_screen {
                            Screen::Home(home) => match home.handle_key(key) {
                                HomeAction::StartSession => {
                                    self.last_session = None;
                                    self.current_screen = Screen::Session(Session::new());
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
                                        self.current_screen = Screen::Session(
                                            Session::resume(ls.started_at, ls.duration_sec, ls.label)
                                        );
                                    }
                                },
                                HomeAction::ViewHistory => {
                                    let month_filter = SessionFilter { since: Some(since_days(30)), tag: None };
                                    let r = self.db.get_sessions(&month_filter).expect(fail_load_history);
                                    self.current_screen = Screen::History(History::new(r));
                                }
                                HomeAction::ViewTags => {
                                    let tags = self.db.get_all_labels_with_counts().expect("failed to load tags");
                                    self.current_screen = Screen::Tags(Tags::new(tags));
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
                                },
                                HistoryAction::QueryLabels(prefix) => {
                                    let suggestions = self.db.get_labels(&prefix).expect(fail_load_label);
                                    hist.update_suggestions(suggestions);
                                },
                                HistoryAction::ExportAllSession => {
                                    use crate::export::{write_sessions_csv, open_dir};
                                    
                                    let cfg = Config::load();
                                    let csv_path = Config::resolved_csv_path(&cfg);
                                    let no_filter = SessionFilter{ since: None, tag: None };  // export all Sessions
                                    let sessions = self.db.get_sessions(&no_filter).expect("failed to load sessions");
                                    write_sessions_csv(&csv_path, &sessions).expect("failed to write sessions to csv");

                                    let staze_path = Config::resolved_staze_path(&cfg);
                                    open_dir(&staze_path)?;
                                }
                                HistoryAction::None => {},
                            },
                            Screen::Tags(tags) => match tags.handle_key(key) {
                                TagsAction::Stop => self.current_screen = Screen::Home(Home::default()),
                                TagsAction::QueryLabels(prefix) => {
                                    let suggestions = self.db.get_labels(&prefix).expect(fail_load_label);
                                    tags.update_suggestions(suggestions);
                                }
                                TagsAction::Delete(label) => {
                                    self.db.delete_label(&label).expect("failed to delete label");
                                    let updated = self.db.get_all_labels_with_counts().expect("failed to reload tags");
                                    tags.update(updated);
                                }
                                TagsAction::Rename { old, new } => {
                                    self.db.rename_label(&old, &new).expect("failed to rename label");
                                    let updated = self.db.get_all_labels_with_counts().expect("failed to reload tags");
                                    tags.update(updated);
                                }
                                TagsAction::None => {}
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
        last_session: None
    }
    .run(terminal))
}
