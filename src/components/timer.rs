use crate::components::Component;
use chrono::{DateTime, Local};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Span, Style};
use ratatui::style::{Modifier, Stylize};
use ratatui::widgets::{Paragraph, Widget};
use ratatui::Frame;
use std::time::Duration;

pub struct Timer {
    start_time: DateTime<Local>,
    duration: Duration,
}

impl Timer {
    pub fn new(seconds: u64) -> Timer {
        Timer {
            start_time: Local::now(),
            duration: Duration::from_secs(seconds),
        }
    }
}

impl Component for Timer {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(self, area);
        Ok(())
    }
}

impl Widget for &mut Timer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let now = Local::now();
        let elapsed = now - self.start_time;
        let elapsed_secs = elapsed.num_seconds() as u64;
        let remaining_secs = self.duration.as_secs() - elapsed_secs;
        let remaining = Duration::from_secs(remaining_secs);
        let remaining_str = format!(
            "剩余时间：{:02}:{:02}:{:02}",
            remaining.as_secs() / 3600,
            remaining.as_secs() / 60,
            remaining.as_secs() % 60
        );
        let [top, _] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        let span = Span::styled(
            remaining_str,
            Style::new()
                .yellow()
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        );
        let paragraph = Paragraph::new(span).right_aligned();
        paragraph.render(top, buf);
    }
}
