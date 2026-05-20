use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Paragraph, Widget},
};

use chrono::DateTime;

use crate::db::SessionRecord;


fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{}h {:02}m", h, m)
}

fn format_timestamp(ts: i64) -> String {
    DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%d-%m-%Y").to_string())
        .unwrap_or_default()
}

pub struct History {
    sessions: Vec<SessionRecord>,
    selected: u8,
    is_label_selected: bool,
    label: String,
    editing: bool,
}

pub enum HistoryAction {
    None,
    Stop,
    Query(u8, String),
}

impl History {
    pub fn new(sessions: Vec<SessionRecord>) -> Self {
        Self {
            selected: 1,  // default view to month
            sessions,
            is_label_selected: false,
            label: "no label selected".to_string(),
            editing: false,
        }
    }

    pub fn update(&mut self, sessions: Vec<SessionRecord>) {
        self.sessions = sessions;
    }

    pub fn handle_key(&mut self, key: KeyCode) -> HistoryAction {
        match key {
            // Navigate between stats views
            KeyCode::Left => {
                self.selected = self.selected.saturating_sub(1);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            KeyCode::Right => {
                self.selected = (self.selected + 1).min(2);
                HistoryAction::Query(self.selected, self.label.clone())
            }
            // Navigate between stats & label
            KeyCode::Down => {
                self.is_label_selected = true;
                HistoryAction::None
            }
            KeyCode::Up => {
                self.is_label_selected = false;
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

impl Widget for &mut History {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main block
        let title = Line::from(" Have you worked well? ".bold());
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<Left/Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let [stats_area, graph_area] = Layout::vertical([
            Constraint::Length(5),
            Constraint::Fill(1),
        ])
        .areas(inner);

        // Label
        let tag_label = if self.editing {
            format!(" < {}_ > ", self.label)
        } else {
            format!(" < {} > ", self.label)
        };
        let style = |i| if self.selected == i { Style::new().reversed() } else { Style::new() };
        let stats_content = vec![
            Line::from(vec![
                " [ Week ] ".set_style(style(0)),
                "   ".into(),
                " [ Month ] ".set_style(style(1)),
                "   ".into(),
                " [ Year ] ".set_style(style(2)),
            ]),
            Line::from(vec![tag_label.into()]),
            Line::from(vec![
                "Total Worked: ".into(),
                format_duration(self.get_total_worked()).bold(),
            ]),
        ];

        Paragraph::new(stats_content)
            .centered()
            .block(Block::bordered().title(" Stats "))
            .render(stats_area, buf);

        if self.sessions.is_empty() {
            Paragraph::new("No session recorded for this period.")
                .centered()
                .block(Block::bordered().title(" Timeline "))
                .render(graph_area, buf);
            return;
        }

        let mut by_day: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
        for s in &self.sessions {
            let day = (s.started_at / 86400) * 86400;  // truncate timestamp to midnight UTC
            *by_day.entry(day).or_insert(0) += s.duration_sec;
        }

        let bars: Vec<Bar> = by_day.iter()
            .map(|(&day, &total)| {
                Bar::default()
                    .value(total as u64)
                    .text_value(format_duration(total))
                    .label(Line::from(format_timestamp(day)))
            })
            .collect();

        BarChart::default()
            .block(Block::bordered().title(" Timeline "))
            .bar_width(11)
            .bar_gap(1)
            .data(BarGroup::default().bars(&bars))
            .render(graph_area, buf);
    }
}
