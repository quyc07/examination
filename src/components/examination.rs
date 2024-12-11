mod question;

use super::Component;
use crate::app::{State, StateHolder};
use crate::components::examination::question::{Questions, SelectQuestion};
use crate::{action::Action, config::Config};
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::Span;
use ratatui::widgets::*;
use ratatui::Frame;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::info;

pub struct Examination {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    questions: Questions<SelectQuestion>,
    question_tx: UnboundedSender<String>,
    answer_rx: UnboundedReceiver<String>,
    state_holder: Arc<Mutex<StateHolder>>,
}

impl Examination {
    pub fn new(
        question_tx: UnboundedSender<String>,
        answer_rx: UnboundedReceiver<String>,
        state_holder: Arc<Mutex<StateHolder>>,
    ) -> Self {
        let mut examination = Self {
            command_tx: None,
            config: Default::default(),
            list_state: Default::default(),
            questions: Default::default(),
            question_tx,
            answer_rx,
            state_holder,
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
        match self.state_holder.lock().unwrap().state {
            State::View => {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
                    KeyCode::Char('l') | KeyCode::Right => {
                        // 弹框请用户输入答案
                        let idx = self.list_state.selected().unwrap();
                        info!("send {idx} to user_input");
                        let user_input = self.questions.0.get_mut(idx).unwrap().user_input.clone();
                        self.question_tx.send(user_input).unwrap();
                    }
                    _ => {}
                }
            }
            State::Input => {}
            State::Submit => {}
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
            self.state_holder.lock().unwrap().set_state(State::View);
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
