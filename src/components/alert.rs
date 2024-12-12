use crate::action::Action;
use crate::app::{Mode, ModeHolder};
use crate::components::area_util::centered_rect;
use crate::components::Component;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, Mutex};

pub struct Alert {
    /// alert message
    msg: String,
    /// 全局状态
    mode_holder: Arc<Mutex<ModeHolder>>,
}

impl Component for Alert {
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        match self.get_state() {
            Mode::Alert => match key.code {
                KeyCode::Enter => Ok(Some(Action::Confirm)),
                KeyCode::Esc => {
                    self.close();
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::Alert(msg) => {
                self.msg = msg;
                self.mode_holder.lock().unwrap().mode = Mode::Alert
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        match self.get_state() {
            Mode::Examination => {}
            Mode::Input => {}
            Mode::Alert => {
                let area = centered_rect(50, 30, area);
                let vertical = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ]);
                let [help_area, alert_area, other] = vertical.areas(area);

                let (msg, style) = (
                    vec!["Press Esc to quit, Press Enter to submit.".into()],
                    Style::default(),
                );
                let text = Text::from(Line::from(msg)).patch_style(style);
                let help_message = Paragraph::new(text);
                frame.render_widget(help_message, help_area);

                let alert = Paragraph::new(self.msg.as_str())
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(alert, alert_area);
            }
        }
        Ok(())
    }
}

impl Alert {
    pub fn new(mode_holder: Arc<Mutex<ModeHolder>>) -> Self {
        Self {
            msg: String::new(),
            mode_holder,
        }
    }

    fn get_state(&self) -> Mode {
        self.mode_holder.lock().unwrap().mode.clone()
    }

    fn close(&mut self) {
        self.mode_holder.lock().unwrap().mode = Mode::Examination;
    }
}
