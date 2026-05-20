use crossterm::event::KeyCode;

use ratatui::{
    buffer::Buffer,
    layout::{Layout, Constraint, Rect},
    style::{Style, Styled, Stylize, Color},
    symbols,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget, Chart, Dataset, Axis, GraphType},
};

use chrono::DateTime;

use crate::db::SessionRecord;


fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    format!("{}h {:02}m", h, m)
}

fn format_timestamp(ts: f64) -> String {
    DateTime::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%d-%m-%Y").to_string())
        .unwrap_or_default()
}

pub struct History {
    sessions: Vec<SessionRecord>,
    selected: u8,
}

pub enum HistoryAction {
    None,
    Stop,
    Query(u8)
}

impl History {
    pub fn new(sessions: Vec<SessionRecord>) -> Self {
        Self {
            selected: 1,  // display month data by default
            sessions: sessions
        }
    }

    pub fn update(&mut self, sessions: Vec<SessionRecord>) {
        self.sessions = sessions;
    }

    pub fn sessions_to_data(&self) -> Vec<(f64, f64)> {
        let data: Vec<(f64, f64)> = self.sessions.iter()
            .map(|s| (s.started_at as f64, s.duration_sec as f64))
            .collect();
        return data
    }

    pub fn handle_key(&mut self, key: KeyCode) -> HistoryAction {
        match key {
            KeyCode::Left => {
                self.selected = (self.selected).saturating_sub(1);
                HistoryAction::Query(self.selected)
            },
            KeyCode::Right => {
                self.selected = (self.selected + 1).min(2);
                HistoryAction::Query(self.selected)
            }
            KeyCode::Char('q') | KeyCode::Esc => HistoryAction::Stop,
            _ => HistoryAction::None,
        }
    }
    
    fn get_total_worked(&self) -> i64 {
        self.sessions.iter().map(|s| s.duration_sec).sum()
    }
}

impl Widget for &History {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main block
        let title = Line::from(" Have you worked well? ".bold());
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<Left/Right>".blue().bold(),
            " Select ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        // Clean visual separation between navigation, stats and graph
        let [stats_area, graph_area] = Layout::vertical([
            Constraint::Length(5),
            Constraint::Fill(1),
        ]).areas(inner);

        let style = |i| if self.selected == i {Style::new().reversed()} else {Style::new()};
        let week_style = style(0);
        let month_style = style(1);
        let year_style = style(2);
        
        let total_duration = self.get_total_worked();

        let stats_content = vec![
            Line::from(vec![
            " [ Week ] ".set_style(week_style),
            "   ".into(),
            " [ Month ] ".set_style(month_style),
            "   ".into(),
            " [ Year ] ".set_style(year_style),
            ]),
            Line::from(vec!["Total Worked:".into(), format_duration(total_duration).bold()]),
            ];
        
        // Stats
        Paragraph::new(stats_content)
            .centered()
            .block(Block::bordered().title(" Stats "))
            .render(stats_area, buf);

        // Graph
        let data = self.sessions_to_data();

        let x_min = data.iter().map(|(x, _)| *x).fold(f64::MAX, f64::min);
        let x_max = data.iter().map(|(x, _)| *x).fold(f64::MIN, f64::max);
        let x_mid = (x_min + x_max) / 2.0;
        let y_max = data.iter().map(|(_, y)| *y).fold(f64::MIN, f64::max);

        let dataset = Dataset::default()
            .name("Sessions")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&data);

        let chart = Chart::new(vec![dataset])
            .x_axis(Axis::default()
                .title("Date")
                .bounds([x_min, x_max])
                .labels(vec![
                    format_timestamp(x_min).bold().into(),
                    format_timestamp(x_mid).bold().into(),
                    format_timestamp(x_max).bold().into(),
                    "today".bold(),
                ]))
            .y_axis(Axis::default()
                .title("Duration")
                .bounds([0.0, y_max])
                .labels(vec!["0".bold(), format_duration((y_max / 2.0) as i64).bold(), format_duration(y_max as i64).bold()]))
            .block(Block::bordered().title(" Timeline "));
        chart.render(graph_area, buf);
    }
}