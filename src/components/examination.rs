mod question;

use super::Component;
use crate::components::examination::question::{Questions, SelectQuestion};
use crate::components::user_input::{InputMode, UserInput};
use crate::{action::Action, config::Config};
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::*;
use ratatui::Frame;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

#[derive(Default)]
pub struct Examination {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    state: ListState,
    questions: Questions<SelectQuestion>,
    question_tx: Option<UnboundedSender<usize>>,
}

impl Examination {
    pub fn new(question_tx: UnboundedSender<usize>) -> Self {
        let mut examination = Self::default();
        examination.state.select_first();
        examination.question_tx = Some(question_tx);
        examination
    }
}

impl Component for Examination {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        info!("Received key event: {:?}", key);
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.state.select_previous(),
            KeyCode::Char('h') | KeyCode::Left => {
                // TODO 关闭弹框，并显示用户输入的答案
            }
            KeyCode::Char('l') | KeyCode::Right => {
                // TODO 弹框请用户输入答案
                let idx = self.state.selected().unwrap();
                info!("send {idx} to user_input");
                self.question_tx.as_ref().unwrap().send(idx)?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "***考试",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        let total_area = sub_rect(0, 60, 0, 100, area);

        let list = List::from_iter(&*self.questions)
            .style(Color::White)
            .highlight_style(Style::default().fg(Color::Green))
            .highlight_symbol("> ")
            .scroll_padding(1)
            .block(block);
        frame.render_stateful_widget(list, total_area, &mut self.state);

        Ok(())
    }
}

pub(crate) fn total_area(frame: &mut Frame) -> Rect {
    Rect::new(0, 0, frame.area().width * 6 / 10, frame.area().height)
}

fn split_rect(percent: u16, r: Rect, direction: Direction) -> (Rect, Rect) {
    let rects = Layout::default()
        .direction(direction)
        .constraints([Constraint::Percentage(percent), Constraint::Fill(1)])
        .split(r);
    (rects[0], rects[1])
}

// ANCHOR: centered_rect
/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
// ANCHOR_END: centered_rect

fn sub_rect(
    percent_x: u16,
    percent_width: u16,
    percent_y: u16,
    percent_high: u16,
    r: Rect,
) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(percent_x),
            Constraint::Percentage(percent_width),
            Constraint::Percentage(100 - percent_x - percent_width),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(percent_y),
            Constraint::Percentage(percent_high),
            Constraint::Percentage(100 - percent_y - percent_high),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
