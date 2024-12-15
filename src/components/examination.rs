mod question;

use super::Component;
use crate::action::ConfirmEvent;
use crate::app::{Mode, ModeHolder};
use crate::components::examination::question::{Questions, SelectQuestion};
use crate::{action::Action, config::Config};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::*;
use ratatui::Frame;
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
    mode_holder: Arc<Mutex<ModeHolder>>,
    score: Option<u16>,
    state: State,
}

#[derive(Eq, PartialEq)]
pub(crate) enum State {
    Ing,
    End,
}

impl Examination {
    pub fn new(
        question_tx: UnboundedSender<String>,
        answer_rx: UnboundedReceiver<String>,
        state_holder: Arc<Mutex<ModeHolder>>,
        config: Config,
    ) -> Self {
        let mut examination = Self {
            command_tx: None,
            config: config.clone(),
            list_state: Default::default(),
            questions: Questions::load(config),
            question_tx,
            answer_rx,
            mode_holder: state_holder,
            score: None,
            state: State::Ing,
        };
        examination.list_state.select_first();
        examination
    }

    fn cal_score(&self) -> u16 {
        self.questions
            .0
            .iter()
            .map(|q| {
                q.user_input
                    .clone()
                    .map(|user_input| {
                        if q.answer.eq_ignore_ascii_case(user_input.as_str()) {
                            q.score
                        } else {
                            0
                        }
                    })
                    .unwrap_or(0)
            })
            .sum::<u16>()
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
        if self.mode_holder.lock().unwrap().mode == Mode::Examination {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Char('l') | KeyCode::Right if self.state == State::Ing => {
                    // 弹框请用户输入答案
                    let idx = self.list_state.selected().unwrap();
                    info!("send {idx} to user_input");
                    let user_input = self
                        .questions
                        .0
                        .get_mut(idx)
                        .unwrap()
                        .user_input
                        .clone()
                        .unwrap_or_default();
                    self.question_tx.send(user_input).unwrap();
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            // 交卷
            Action::Submit => {
                // 判断是否全部题目都已经做完，否则弹框提示
                if self.questions.0.iter().any(|q| q.user_input.is_none()) {
                    return Ok(Some(Action::Alert(
                        "还有题目未做完，是否确认交卷？".to_string(),
                        ConfirmEvent::Submit,
                    )));
                }
                Ok(None)
            }
            Action::Confirm(ConfirmEvent::Submit) => {
                self.mode_holder.lock().unwrap().mode = Mode::Examination;
                // 计算得分
                let score = self.cal_score();
                self.score = Some(score);
                Ok(Some(Action::Alert(
                    format!("您的最终得分是{score}"),
                    ConfirmEvent::Score,
                )))
            }
            Action::Confirm(ConfirmEvent::Score) => {
                self.state = State::End;
                self.mode_holder.lock().unwrap().mode = Mode::Examination;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Ok(answer) = self.answer_rx.try_recv() {
            self.mode_holder.lock().unwrap().set_mode(Mode::Examination);
            let question = self
                .questions
                .0
                .get_mut(self.list_state.selected().unwrap())
                .unwrap();
            question.user_input = Some(answer);
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "***考试",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        let texts = self
            .questions
            .iter()
            .enumerate()
            .map(|(i, q)| q.convert_lines(&self.state, i))
            .collect::<Vec<Text>>();

        let list = match self.state {
            State::Ing => List::from_iter(texts)
                .style(Color::Gray)
                .highlight_style(Style::default().fg(Color::LightBlue))
                .highlight_symbol("> ")
                .scroll_padding(1)
                .block(block),
            State::End => List::from_iter(texts)
                .style(Color::Gray)
                .scroll_padding(1)
                .block(block),
        };
        frame.render_stateful_widget(list, area, &mut self.list_state);
        Ok(())
    }
}
