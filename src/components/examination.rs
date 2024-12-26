mod question;

use super::Component;
use crate::action::ConfirmEvent;
use crate::app::{Mode, ModeHolder};
use crate::components::examination::question::{
    FillIn, Judge, MultiSelect, Question, SingleSelect,
};
use crate::{action::Action, config::Config};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::*;
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::info;

pub struct Examination {
    examination_config: ExaminationConfig,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    questions: Vec<QuestionEnum>,
    question_tx: UnboundedSender<QuestionEnum>,
    answer_rx: UnboundedReceiver<QuestionEnum>,
    mode_holder: Arc<Mutex<ModeHolder>>,
    score: Option<u16>,
    state: State,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExaminationConfig {
    name: String,
    duration: u64,
    single_select: usize,
    multi_select: usize,
    judge: usize,
    fill_in: usize,
}

impl ExaminationConfig {
    pub fn duration(&self) -> u64 {
        self.duration
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum State {
    Ing,
    End,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum QuestionEnum {
    SingleSelect(SingleSelect),
    MultiSelect(MultiSelect),
    Judge(Judge),
    FillIn(FillIn),
}

impl Examination {
    pub fn new(
        question_tx: UnboundedSender<QuestionEnum>,
        answer_rx: UnboundedReceiver<QuestionEnum>,
        state_holder: Arc<Mutex<ModeHolder>>,
        config: Config,
        ec: ExaminationConfig,
    ) -> Self {
        let mut examination = Self {
            examination_config: ec.clone(),
            command_tx: None,
            config: config.clone(),
            list_state: Default::default(),
            questions: QuestionEnum::load(config, ec),
            question_tx,
            answer_rx,
            mode_holder: state_holder,
            score: None,
            state: State::Ing,
        };
        examination.list_state.select_first();
        examination
    }

    pub(crate) fn load(config: Config) -> ExaminationConfig {
        let mut ec = String::new();
        File::open(config.config.data_dir.join("examination.json"))
            .unwrap()
            .read_to_string(&mut ec)
            .expect("Fail to load question!");
        serde_json::from_slice::<ExaminationConfig>(ec.as_ref()).unwrap()
    }

    fn cal_score(&self) -> u16 {
        self.questions
            .iter()
            .map(|q| match q {
                QuestionEnum::SingleSelect(q) => q.cal_score(),
                QuestionEnum::MultiSelect(q) => q.cal_score(),
                QuestionEnum::Judge(q) => q.cal_score(),
                QuestionEnum::FillIn(q) => q.cal_score(),
            })
            .sum::<u16>()
    }

    fn handle_submit(&mut self) -> Result<Option<Action>> {
        self.mode_holder.lock().unwrap().mode = Mode::Examination;
        // 计算得分
        let score = self.cal_score();
        self.score = Some(score);
        Ok(Some(Action::Alert(
            format!("您的最终得分是{score}"),
            ConfirmEvent::Score,
        )))
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
                    let q = self.questions.get(idx).unwrap();
                    self.question_tx.send(q.clone()).unwrap();
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
                if self.questions.iter().all(|q| q.answered()) {
                    self.handle_submit()
                } else {
                    Ok(Some(Action::Alert(
                        "还有题目未做完，是否确认交卷？".to_string(),
                        ConfirmEvent::Submit,
                    )))
                }
            }
            Action::Confirm(ConfirmEvent::Submit) => self.handle_submit(),
            Action::Confirm(ConfirmEvent::Score) => {
                self.state = State::End;
                self.mode_holder.lock().unwrap().mode = Mode::Examination;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if let Ok(q) = self.answer_rx.try_recv() {
            self.mode_holder.lock().unwrap().set_mode(Mode::Examination);
            self.questions[self.list_state.selected().unwrap()] = q;
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                &self.examination_config.name,
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        let texts = self
            .questions
            .iter()
            .enumerate()
            .map(|(i, q)| q.convert_text(self.state, i))
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
