use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, ListState, Paragraph, StatefulWidget, Widget},
};

use chrono::DateTime;

use crate::db::SessionRecord;
use crate::label_input::{InputEvent, LabelInput};


fn format_duration(secs: i64, hours_width: usize) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{:0width$}h {:02}m", h, m, width = hours_width)
}

fn format_date_short(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%d-%m").to_string())
        .unwrap_or_default()
}

fn format_month_label(month_key: i64) -> String {
    let names = ["Jan","Feb","Mar","Apr","May","Jun",
                 "Jul","Aug","Sep","Oct","Nov","Dec"];
    let month = (month_key % 100) as usize; // key = YYYY*100+MM (1-indexed)
    names[month - 1].to_string()
}

pub struct History {
    sessions: Vec<SessionRecord>,
    selected: u8,
    is_cursor_on_label: bool,
    label: Option<String>,
    input: LabelInput,
}

pub enum HistoryAction {
    None,
    Stop,
    Query(u8, Option<String>),
    QueryLabels(String),
    ExportAllSession,
}

impl History {
    pub fn new(sessions: Vec<SessionRecord>) -> Self {
        Self {
            selected: 1,
            sessions,
            is_cursor_on_label: false,
            label: None,
            input: LabelInput::default(),
        }
    }

    pub fn update(&mut self, sessions: Vec<SessionRecord>) {
        self.sessions = sessions;
    }

    pub fn is_typing(&self) -> bool {
        self.input.is_active()
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.input.update_suggestions(suggestions);
    }

    fn clear_filter(&mut self) -> HistoryAction {
        match self.label.take() {
            Some(_) => HistoryAction::Query(self.selected, None),
            None => HistoryAction::None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> HistoryAction {
        // Label search: the chart only updates once a label is confirmed
        if self.input.is_active() {
            return match self.input.handle_key(key) {
                InputEvent::Query(prefix) => HistoryAction::QueryLabels(prefix),
                InputEvent::Submit(label) => {
                    self.label = label;
                    HistoryAction::Query(self.selected, self.label.clone())
                }
                InputEvent::Cancel => self.clear_filter(),
                InputEvent::None => HistoryAction::None,
            };
        }
        match key {
            // Label search: `/` from anywhere, or Enter on the label row
            KeyCode::Char('/') => {
                self.is_cursor_on_label = true;
                HistoryAction::QueryLabels(self.input.open(None))
            }
            KeyCode::Enter if self.is_cursor_on_label => HistoryAction::QueryLabels(self.input.open(None)),
            // Row navigation
            KeyCode::Down | KeyCode::Char('j') => {
                self.is_cursor_on_label = true;
                HistoryAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.is_cursor_on_label = false;
                HistoryAction::None
            }
            // Period navigation
            KeyCode::Left | KeyCode::Char('h')=> {
                self.selected = self.selected.saturating_sub(1);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected = (self.selected + 1).min(2);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            KeyCode::Char('e') => {
                HistoryAction::ExportAllSession
            }
            KeyCode::Esc if self.label.is_some() => self.clear_filter(),
            // Exit
            KeyCode::Char('q') | KeyCode::Esc => HistoryAction::Stop,
            _ => HistoryAction::None,
        }
    }

    fn get_total_worked(&self) -> i64 {
        self.sessions.iter().map(|s| s.duration_sec).sum()
    }
}

impl StatefulWidget for &mut History {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut ListState) {
        let title = Line::from(" Have you worked well? ".bold());
        let instructions = if self.input.is_active() {
            LabelInput::instructions()
        } else {
            let mut instruction_spans = vec![
                " Navigate ".into(),
                "<Arrows> ; <h/j/k/l>".blue().bold(),
                " Search ".into(),
                "</> ; <Enter>".blue().bold(),
            ];
            if self.label.is_some() {
                instruction_spans.extend([
                    " Clear filter ".into(),
                    "<Esc> ".blue().bold(),
                ]);
            }
            instruction_spans.extend([
                " Export (.csv) ".into(),
                "<E>".blue().bold(),
                " Back ".into(),
                "<Esc> ".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ]);
            Line::from(instruction_spans)
        };

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let [stats_area, suggestions_area, graph_area] = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(self.input.dropdown_height()),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let style = |i| if self.selected == i && !self.is_cursor_on_label { Style::new().reversed() } else { Style::new() };
        let label_style = if self.is_cursor_on_label { Style::new().reversed() } else { Style::new() };
        let tag_label = self.input.display(self.label.as_deref(), " [ all labels ] ");

        let total_hours_length = (self.get_total_worked() / 3600).to_string().len().max(1);

        let stats_content = vec![
            Line::from(vec![
                " [ Week ] ".set_style(style(0)),
                "   ".into(),
                " [ Month ] ".set_style(style(1)),
                "   ".into(),
                " [ Year ] ".set_style(style(2)),
            ]),
            Line::from(vec![tag_label.set_style(label_style)]),
            Line::from(vec![
                "Total Worked: ".into(),
                format_duration(self.get_total_worked(), total_hours_length).bold(),
            ]),
        ];

        Paragraph::new(stats_content)
            .centered()
            .block(Block::bordered().title(" Stats "))
            .render(stats_area, buf);

        self.input.render_dropdown(suggestions_area, buf, " Filter by label ");

        if self.sessions.is_empty() {
            Paragraph::new("No session recorded for this period.")
                .centered()
                .block(Block::bordered().title(" Timeline "))
                .render(graph_area, buf);
            return;
        }

        use chrono::Datelike;

        let totals: Vec<(String, i64)> = if self.selected == 2 {
            let mut by_month: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
            for s in &self.sessions {
                if let Some(dt) = DateTime::from_timestamp(s.started_at, 0) {
                    let key = dt.year() as i64 * 100 + dt.month() as i64;
                    *by_month.entry(key).or_insert(0) += s.duration_sec;
                }
            }
            by_month.iter()
            .map(|(key, total)| {
                (format_month_label(*key), *total)
            }).collect()
        } else {
            let mut by_day: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
            for s in &self.sessions {
                let day = (s.started_at / 86400) * 86400;
                *by_day.entry(day).or_insert(0) += s.duration_sec;
            }
            by_day.iter()
            .map(|(day, total)|{
                (format_date_short(*day), *total)
            }).collect()
        };

        // Scale the bars based on the maximum value to ensure they fit in the chart area
        let real_max = totals.iter().map(|(_, total)| total).max().unwrap_or(&0);
        let ceiling = (*real_max as f64 * 1.1).ceil(); // 10% padding to the maximum value to ensure the tallest bar fits in the chart area
        let inner_height = graph_area.height.saturating_sub(2); // borders
        let min_rows = 2u64; // enough for bar + text_value line
        let floor = if inner_height > 0 {
            (ceiling * min_rows as f64 / inner_height as f64).ceil()
        } else {
            0.0
        };


        let max_hours = real_max / 3600;
        let hours_width = max_hours.to_string().len().max(1);

        let max_text_length = totals.iter()
        .map(|(_, t)| format_duration(*t, hours_width).len())
        .max()
        .unwrap_or(6) as u16;
        
        let bars: Vec<Bar> = totals.iter()
        .map(|(label, t)| {
            Bar::default()
            .value((*t as u64).max(floor as u64))
            .text_value(format_duration(*t, hours_width))
            .label(Line::from(label.clone()))
        }).collect();
        
        BarChart::default()
            .block(Block::bordered().title(" Timeline "))
            .bar_width(max_text_length)
            .bar_gap(1)
            .max(ceiling as u64)
            .data(BarGroup::default().bars(&bars))
            .render(graph_area, buf);
    }
}
