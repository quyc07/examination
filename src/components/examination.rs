mod question;

use super::Component;
use crate::components::area_util::{sub_rect, user_input_area};
use crate::components::examination::question::{Questions, SelectQuestion};
use crate::components::user_input::UserInput;
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
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::id;
use tracing::info;

#[derive(Eq, PartialEq)]
enum State {
    View,
    Input,
}

pub struct Examination {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    questions: Questions<SelectQuestion>,
    question_tx: UnboundedSender<String>,
    answer_rx: UnboundedReceiver<String>,
    state: State,
}

impl Examination {
    pub fn new(question_tx: UnboundedSender<String>, answer_rx: UnboundedReceiver<String>) -> Self {
        let mut examination = Self {
            command_tx: None,
            config: Default::default(),
            list_state: Default::default(),
            questions: Default::default(),
            question_tx,
            answer_rx,
            state: State::View,
        };
        examination.list_state.select_first();
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
        match self.state {
            State::View => {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
                    KeyCode::Char('h') | KeyCode::Left => {
                        // TODO 关闭弹框，并显示用户输入的答案
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        // TODO 弹框请用户输入答案
                        let idx = self.list_state.selected().unwrap();
                        info!("send {idx} to user_input");
                        let user_input = self.questions.0.get_mut(idx).unwrap().user_input.clone();
                        self.question_tx.send(user_input)?;
                        self.state = State::Input
                    }
                    _ => {}
                }
            }
            State::Input => {}
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
        if let Ok(answer) = self.answer_rx.try_recv() {
            self.state = State::View;
            let question = self
                .questions
                .0
                .get_mut(self.list_state.selected().unwrap())
                .unwrap();
            question.user_input = answer;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "***考试",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        let list = List::from_iter(&*self.questions)
            .style(Color::White)
            .highlight_style(Style::default().fg(Color::Green))
            .highlight_symbol("> ")
            .scroll_padding(1)
            .block(block);
        frame.render_stateful_widget(list, area, &mut self.list_state);
        Ok(())
    }
}

pub(crate) fn total_area(frame: &mut Frame) -> Rect {
    Rect::new(0, 0, frame.area().width * 6 / 10, frame.area().height)
}
