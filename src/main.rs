use std::io;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};

mod config;
mod home;
mod session;
mod history;
mod db;
mod tags;
mod staz;

use staz::{Mood, Staz};

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

fn since_days(days: u64) -> i64 {
    let cutoff = SystemTime::now() - Duration::from_secs(days * 86400);
    cutoff.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

/// Width/height (in cells) of multi-line ascii art.
fn art_size(art: &str) -> (u16, u16) {
    let h = art.lines().count() as u16;
    let w = art.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    (w, h)
}

/// A `Rect` the size of `art`, centered within `area`.
fn centered(area: Rect, art: &str) -> Rect {
    let (w, h) = art_size(art);
    let (w, h) = (w.min(area.width), h.min(area.height));
    Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h }
}

/// A `Rect` the size of `art`, anchored to the bottom-left of `area`.
fn bottom_left(area: Rect, art: &str) -> Rect {
    let (w, h) = art_size(art);
    let (w, h) = (w.min(area.width), h.min(area.height));
    Rect { x: area.x, y: area.bottom() - h, width: w, height: h }
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
    staz: Staz,
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
        self.staz.tick();
        let area = frame.area();
        match &mut self.current_screen {
            Screen::Home(home) => frame.render_widget(home, area),
            Screen::Session(session) => frame.render_stateful_widget(session, area, &mut ListState::default()),
            Screen::History(history) => frame.render_stateful_widget(history, area, &mut ListState::default()),
            Screen::Tags(tags) => frame.render_stateful_widget(tags, area, &mut ListState::default()),
        }

        // Staz overlay: centered on Home, bottom-left in Session, hidden elsewhere.
        let spot = match &self.current_screen {
            Screen::Home(_) => Some(centered(area, self.staz.frame())),
            Screen::Session(_) => Some(bottom_left(area, self.staz.frame())),
            _ => None,
        };
        if let Some(spot) = spot {
            self.staz.render(spot, frame.buffer_mut());
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let fail_load_history = "failed to load history";
        let fail_load_label = "failed to fetch labels";

        let frame_budget = Duration::from_millis(33);  // 30 fps rendering
        let mut next_deadline = Instant::now();
        
        let now = Instant::now();
        let timeout = next_deadline.saturating_duration_since(now);
        
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    let editing = matches!(&self.current_screen, Screen::Tags(t) if t.is_editing());
                    match key_event.code {
                        KeyCode::Char('q') if !editing => self.exit = true,
                        key => match &mut self.current_screen {
                            Screen::Home(home) => match home.handle_key(key) {
                                HomeAction::StartSession => {
                                    self.last_session = None;
                                    let suggestions = self.db.get_labels("").expect(fail_load_label);
                                    self.current_screen = Screen::Session(Session::new(suggestions));
                                    self.staz.set(Mood::Working);
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
                                        self.staz.set(Mood::Working);
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
                                    self.staz.set(Mood::Idle);
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
                            },
                            Screen::Tags(tags) => match tags.handle_key(key) {
                                TagsAction::Stop => self.current_screen = Screen::Home(Home::default()),
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
        if Instant::now() >= next_deadline {
            next_deadline += frame_budget;
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
        staz: Staz::new()
    }
    .run(terminal))
}
