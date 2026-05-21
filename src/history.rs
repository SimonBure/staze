use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use chrono::DateTime;

use crate::db::SessionRecord;


fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{}h {:02}m", h, m)
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
    picking_label: bool,
    label: Option<String>,
    suggestions: Vec<String>,
    suggestion_state: ListState,
}

pub enum HistoryAction {
    None,
    Stop,
    Query(u8, Option<String>),
}

impl History {
    pub fn new(sessions: Vec<SessionRecord>) -> Self {
        Self {
            selected: 1,
            sessions,
            is_cursor_on_label: false,
            picking_label: false,
            label: None,
            suggestions: vec![],
            suggestion_state: ListState::default(),
        }
    }

    pub fn update(&mut self, sessions: Vec<SessionRecord>) {
        self.sessions = sessions;
    }

    pub fn update_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        self.suggestion_state.select(None);
    }

    pub fn handle_key(&mut self, key: KeyCode) -> HistoryAction {
        match key {
            // Open suggestions dropdown
            KeyCode::Enter if self.is_cursor_on_label && !self.picking_label => {
                self.picking_label = true;
                HistoryAction::None
            }
            // Navigate suggestions
            KeyCode::Down if self.picking_label => {
                self.suggestion_state.select_next();
                HistoryAction::None
            }
            KeyCode::Up if self.picking_label => {
                self.suggestion_state.select_previous();
                HistoryAction::None
            }
            // Pick a suggestion
            KeyCode::Enter if self.picking_label => {
                if let Some(i) = self.suggestion_state.selected()
                    && let Some(picked) = self.suggestions.get(i) {
                        self.label = Some(picked.clone());
                    }
                self.picking_label = false;
                self.suggestion_state.select(None);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            // Clear label filter
            KeyCode::Char('c') if self.label.is_some() => {
                self.picking_label = false;
                self.label = None;
                self.suggestion_state.select(None);
                HistoryAction::Query(self.selected, None)
            }
            // Period navigation
            KeyCode::Left => {
                self.selected = self.selected.saturating_sub(1);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            KeyCode::Right => {
                self.selected = (self.selected + 1).min(2);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            // Row navigation
            KeyCode::Down => {
                self.is_cursor_on_label = true;
                HistoryAction::None
            }
            KeyCode::Up => {
                self.is_cursor_on_label = false;
                HistoryAction::None
            }
            // Close dropdown without picking (q while open)
            KeyCode::Char('q') if self.picking_label => {
                self.picking_label = false;
                self.suggestion_state.select(None);
                HistoryAction::None
            }
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
        let mut instruction_spans = vec![
            " Navigate ".into(),
            "<Left/Right>".blue().bold(),
        ];
        if self.label.is_some() {
            instruction_spans.extend([
                " Clear filter ".into(),
                "<C> ".blue().bold(),
            ]);
        }
        instruction_spans.extend([
            " Back ".into(),
            "<Esc> ".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let instructions = Line::from(instruction_spans);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let [stats_area, suggestions_area, graph_area] = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(if self.picking_label && !self.suggestions.is_empty() {
                self.suggestions.len() as u16 + 2
            } else { 0 }),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let style = |i| if self.selected == i && !self.is_cursor_on_label { Style::new().reversed() } else { Style::new() };
        let label_style = if self.is_cursor_on_label { Style::new().reversed() } else { Style::new() };
        let tag_label = match &self.label {
            Some(l) => format!(" < {} > ", l),
            None    => " [ all labels ] ".to_string(),
        };

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
                format_duration(self.get_total_worked()).bold(),
            ]),
        ];

        Paragraph::new(stats_content)
            .centered()
            .block(Block::bordered().title(" Stats "))
            .render(stats_area, buf);

        if self.picking_label && !self.suggestions.is_empty() {
            let items: Vec<ListItem> = self.suggestions.iter()
                .map(|l| ListItem::new(l.as_str()))
                .collect();
            let list = List::new(items)
                .highlight_style(Style::new().reversed())
                .block(Block::bordered().title(" Filter by label "));
            StatefulWidget::render(list, suggestions_area, buf, &mut self.suggestion_state);
        }

        if self.sessions.is_empty() {
            Paragraph::new("No session recorded for this period.")
                .centered()
                .block(Block::bordered().title(" Timeline "))
                .render(graph_area, buf);
            return;
        }

        use chrono::Datelike;

        let bars: Vec<Bar> = if self.selected == 2 {
            let mut by_month: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
            for s in &self.sessions {
                if let Some(dt) = DateTime::from_timestamp(s.started_at, 0) {
                    let key = dt.year() as i64 * 100 + dt.month() as i64;
                    *by_month.entry(key).or_insert(0) += s.duration_sec;
                }
            }
            by_month.iter()
                .map(|(&key, &total)| {
                    Bar::default()
                        .value(total as u64)
                        .text_value(format_duration(total))
                        .label(Line::from(format_month_label(key)))
                })
                .collect()
        } else {
            let mut by_day: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
            for s in &self.sessions {
                let day = (s.started_at / 86400) * 86400;
                *by_day.entry(day).or_insert(0) += s.duration_sec;
            }
            by_day.iter()
                .map(|(&day, &total)| {
                    Bar::default()
                        .value(total as u64)
                        .text_value(format_duration(total))
                        .label(Line::from(format_date_short(day)))
                })
                .collect()
        };

        BarChart::default()
            .block(Block::bordered().title(" Timeline "))
            .bar_width(6)
            .bar_gap(1)
            .data(BarGroup::default().bars(&bars))
            .render(graph_area, buf);
    }
}
