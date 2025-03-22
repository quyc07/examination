mod question;

use super::Component;
use crate::action::ConfirmEvent;
use crate::app::{Mode, ModeHolder, ModeHolderLock};
use crate::components::examination::question::{
    FillIn, Judge, MultiSelect, Question, SingleSelect,
};
use crate::{action::Action, config::Config};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use linked_hash_map::LinkedHashMap;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Alignment, Layout, Rect};
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Examination {
    examination_config: ExaminationConfig,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    list_state: ListState,
    questions: LinkedHashMap<QuestionType, Vec<QuestionEnum>>,
    question_tx: UnboundedSender<QuestionEnum>,
    answer_rx: UnboundedReceiver<QuestionEnum>,
    mode_holder: ModeHolderLock,
    score: Option<u16>,
    state: State,
    selected_tab: QuestionType,
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

#[derive(Eq, PartialEq, Copy, Clone, Deserialize, Serialize, Hash, Display, FromRepr, EnumIter)]
pub enum QuestionType {
    #[strum(to_string = "单选题")]
    SingleSelect,
    #[strum(to_string = "多选题")]
    MultiSelect,
    #[strum(to_string = "判断题")]
    Judge,
    #[strum(to_string = "填空题")]
    FillIn,
}

struct QuestionTabInner {
    questions: Vec<QuestionEnum>,
    state: State,
    list_state: ListState,
}

impl Widget for QuestionTabInner {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
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
                .scroll_padding(1),
            State::End => List::from_iter(texts).style(Color::Gray).scroll_padding(1),
        };
        let mut state = self.list_state.clone();
        StatefulWidget::render(list, area, buf, &mut state)
    }
}

impl QuestionType {
    /// Get the previous tab, if there is no previous tab return the current tab.
    pub(crate) fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    pub(crate) fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    /// Return tab's name as a styled `Line`
    pub(crate) fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    pub(crate) const fn palette(self) -> tailwind::Palette {
        match self {
            Self::SingleSelect => tailwind::BLUE,
            Self::MultiSelect => tailwind::EMERALD,
            Self::Judge => tailwind::INDIGO,
            Self::FillIn => tailwind::RED,
        }
    }
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
        let type_2_questions = QuestionEnum::load(config.clone(), ec.clone());
        let question_type = type_2_questions
            .iter()
            .next()
            .map(|(k, _)| *k)
            .expect("未能正确加载试卷！");
        let mut examination = Self {
            examination_config: ec.clone(),
            command_tx: None,
            config: config.clone(),
            list_state: Default::default(),
            questions: type_2_questions,
            question_tx,
            answer_rx,
            mode_holder: ModeHolderLock(state_holder),
            score: None,
            state: State::Ing,
            selected_tab: question_type,
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

    pub fn current_questions(&self) -> Vec<QuestionEnum> {
        self.questions
            .get(&self.selected_tab)
            .expect("Fail to get questions!")
            .clone()
    }

    fn cal_score(&self) -> u16 {
        self.questions
            .values()
            .map(|qs| {
                qs.iter()
                    .map(|q| match q {
                        QuestionEnum::SingleSelect(q) => q.cal_score(),
                        QuestionEnum::MultiSelect(q) => q.cal_score(),
                        QuestionEnum::Judge(q) => q.cal_score(),
                        QuestionEnum::FillIn(q) => q.cal_score(),
                    })
                    .sum::<u16>()
            })
            .sum()
    }

    fn handle_submit(&mut self) -> Result<Option<Action>> {
        self.mode_holder.set_mode(Mode::Examination);
        // 计算得分
        let score = self.cal_score();
        self.score = Some(score);
        Ok(Some(Action::Alert(
            format!("您的最终得分是{score}"),
            ConfirmEvent::Score,
        )))
    }

    pub fn next_tab(&mut self) {
        self.list_state.select_first();
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.list_state.select_first();
        self.selected_tab = self.selected_tab.previous();
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        // let titles = self.questions.keys().map(QuestionType::title);
        let titles = QuestionType::iter().map(QuestionType::title);
        let highlight_style = (Color::default(), self.selected_tab.palette().c700);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
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
        if self.mode_holder.get_mode() == Mode::Examination {
            match key.code {
                KeyCode::Down => self.list_state.select_next(),
                KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Enter if self.state == State::Ing => {
                    // 弹框请用户输入答案
                    let idx = self.list_state.selected().unwrap();
                    let current_questions = self.current_questions();
                    let q = current_questions.get(idx).unwrap();
                    self.question_tx.send(q.clone())?;
                }
                KeyCode::Right => self.next_tab(),
                KeyCode::Left => self.previous_tab(),
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
                if self
                    .questions
                    .iter()
                    .all(|q| q.1.iter().all(|q| q.answered()))
                {
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
                self.mode_holder.set_mode(Mode::Examination);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(&mut *self, area);
        Ok(())
    }
}

impl Widget for &mut Examination {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Ok(q) = self.answer_rx.try_recv() {
            self.mode_holder.set_mode(Mode::Examination);
            let current_questions = self.questions.get_mut(&self.selected_tab).unwrap();
            current_questions[self.list_state.selected().unwrap()] = q;
        }

        let vertical = Layout::vertical([Length(1), Length(1), Min(0), Length(1)]);
        let [title_area, tab_area, inner_area, footer_area] = vertical.areas(area);
        Paragraph::new(self.examination_config.name.clone())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .render(title_area, buf);
        self.render_tabs(tab_area, buf);
        let question_tab_inner = QuestionTabInner {
            questions: self.current_questions(),
            state: self.state,
            list_state: self.list_state.clone(),
        };
        question_tab_inner.render(inner_area, buf);
        render_footer(footer_area, buf);
    }
}

pub fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::raw("◄ ► to change tab | Ctrl+c to quit | Enter to write answer")
        .centered()
        .render(area, buf);
}
