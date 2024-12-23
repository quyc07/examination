use crate::components::examination::{QuestionEnum, State};
use crate::config::Config;
use ratatui::prelude::{Line, Text};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::sync::LazyLock;

pub trait Question {
    fn convert_text(&self, state: State, q_index: usize) -> Text<'_>;

    fn option_style(
        &self,
        state: State,
        i: usize,
        user_input_idx: Option<Vec<usize>>,
        answer_idx: Vec<usize>,
    ) -> Style {
        match user_input_idx {
            None => *DEFAULT_STYLE,
            Some(user_input_idx) => match state {
                State::Ing => {
                    if user_input_idx.contains(&i) {
                        *SELECT_STYLE
                    } else {
                        *DEFAULT_STYLE
                    }
                }
                State::End => {
                    if answer_idx.contains(&i) && user_input_idx.contains(&i) {
                        *RIGHT_STYLE
                    } else if user_input_idx.contains(&i) {
                        *WRONG_STYLE
                    } else if answer_idx.contains(&i) {
                        *RIGHT_STYLE
                    } else {
                        *DEFAULT_STYLE
                    }
                }
            },
        }
    }

    fn cal_score(&self) -> u16 {
        self.user_input()
            .clone()
            .map(|user_input| {
                let user_input_set = user_input
                    .chars()
                    .map(|c| c.to_lowercase().to_string())
                    .collect::<HashSet<_>>();
                let answer_set = self
                    .answer()
                    .chars()
                    .map(|c| c.to_lowercase().to_string())
                    .collect::<HashSet<_>>();
                if user_input_set == answer_set {
                    self.score()
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn user_input(&self) -> Option<String>;

    fn answer(&self) -> String;

    fn score(&self) -> u16;

    fn answered(&self) -> bool;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SingleSelect {
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
    pub user_input: Option<String>,
    pub score: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MultiSelect {
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
    pub user_input: Option<String>,
    pub score: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Judge {
    pub question: String,
    pub answer: String,
    pub user_input: Option<String>,
    pub score: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FillIn {
    pub question: String,
    pub items: Vec<FillInItem>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FillInItem {
    pub answer: String,
    pub user_input: Option<String>,
    pub score: u16,
}

impl QuestionEnum {
    pub(crate) fn load(config: Config) -> Vec<QuestionEnum> {
        let mut questions = String::new();
        File::open(config.config.data_dir.join("question.json"))
            .unwrap()
            .read_to_string(&mut questions)
            .expect("Fail to load question!");
        serde_json::from_slice::<Vec<QuestionEnum>>(questions.as_ref()).unwrap()
    }

    pub fn convert_text(&self, state: State, q_index: usize) -> Text<'_> {
        match self {
            QuestionEnum::SingleSelect(q) => q.convert_text(state, q_index),
            QuestionEnum::MultiSelect(q) => q.convert_text(state, q_index),
            QuestionEnum::Judge(q) => q.convert_text(state, q_index),
            QuestionEnum::FillIn(q) => q.convert_text(state, q_index),
        }
    }

    pub fn user_input(&self) -> Vec<Option<String>> {
        match self {
            QuestionEnum::SingleSelect(q) => vec![q.user_input()],
            QuestionEnum::MultiSelect(q) => vec![q.user_input()],
            QuestionEnum::Judge(q) => vec![q.user_input()],
            QuestionEnum::FillIn(q) => q.user_input(),
        }
    }

    pub fn answered(&self) -> bool {
        match self {
            QuestionEnum::SingleSelect(q) => q.answered(),
            QuestionEnum::MultiSelect(q) => q.answered(),
            QuestionEnum::Judge(q) => q.answered(),
            QuestionEnum::FillIn(q) => q.answered(),
        }
    }

    pub fn set_user_input(&mut self, user_input: Vec<Option<String>>) {
        match self {
            QuestionEnum::SingleSelect(q) => {
                q.user_input = user_input[0].clone();
            }
            QuestionEnum::MultiSelect(q) => {
                q.user_input = user_input[0].clone();
            }
            QuestionEnum::Judge(q) => {
                q.user_input = user_input[0].clone();
            }
            QuestionEnum::FillIn(q) => {
                q.items
                    .iter_mut()
                    .zip(user_input.iter())
                    .for_each(|(item, user_input)| item.user_input = user_input.clone());
            }
        }
    }

    pub fn input_size(&self) -> usize {
        match self {
            QuestionEnum::SingleSelect(_) => 1,
            QuestionEnum::MultiSelect(_) => 1,
            QuestionEnum::Judge(_) => 1,
            QuestionEnum::FillIn(q) => q.items.len(),
        }
    }
}

static DEFAULT_STYLE: LazyLock<Style, fn() -> Style> = LazyLock::new(Style::default);
static SELECT_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Yellow));
static RIGHT_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Green));
static WRONG_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Red));

impl Question for SingleSelect {
    fn convert_text(&self, state: State, q_index: usize) -> Text<'_> {
        let mut lines = vec![];
        let mut question = self.question.clone();
        if let Some(user_input) = &self.user_input {
            let answer = format!("（{}）", user_input);
            question = question.replace("（ ）", answer.as_str());
        }
        lines.push(Line::from(format!("{}: {question}", q_index + 1)));
        for (i, option) in self.options.iter().enumerate() {
            let user_input_idx = self
                .user_input
                .clone()
                .and_then(|user_input| to_idx(user_input.as_str()).map(|i| vec![i]));
            let answer_idx = to_idx(self.answer.clone().as_str())
                .map(|i| vec![i])
                .unwrap();
            let style = self.option_style(state, i, user_input_idx, answer_idx);
            lines.push(Line::from(format!("  {option}")).style(style));
        }
        Text::from(lines)
    }

    fn user_input(&self) -> Option<String> {
        self.user_input.clone()
    }

    fn answer(&self) -> String {
        self.answer.clone()
    }

    fn score(&self) -> u16 {
        self.score
    }

    fn answered(&self) -> bool {
        self.user_input.is_some()
    }
}

impl Question for MultiSelect {
    fn convert_text(&self, state: State, q_index: usize) -> Text<'_> {
        let mut lines = vec![];
        let mut question = self.question.clone();
        if let Some(user_input) = &self.user_input {
            let answer = format!("（{}）", user_input);
            question = question.replace("（ ）", answer.as_str());
        }
        lines.push(Line::from(format!("{}: {question}", q_index + 1)));
        for (i, option) in self.options.iter().enumerate() {
            let user_input_idx = self.user_input.clone().map(|user_input| {
                user_input
                    .chars()
                    .filter_map(|c| to_idx(c.to_string().as_str()))
                    .collect()
            });
            let answer_idx = self
                .answer
                .clone()
                .chars()
                .filter_map(|c| to_idx(c.to_string().as_str()))
                .collect();
            let style = self.option_style(state, i, user_input_idx, answer_idx);
            lines.push(Line::from(format!("  {option}")).style(style));
        }
        Text::from(lines)
    }

    fn user_input(&self) -> Option<String> {
        self.user_input.clone()
    }

    fn answer(&self) -> String {
        self.answer.clone()
    }

    fn score(&self) -> u16 {
        self.score
    }

    fn answered(&self) -> bool {
        self.user_input.is_some()
    }
}

impl Question for Judge {
    fn convert_text(&self, state: State, q_index: usize) -> Text<'_> {
        let mut lines = vec![];
        let question = self.question.clone();
        if let Some(user_input) = &self.user_input {
            let pattern_en = Regex::new(r"\(\s*\)|\(\)").unwrap();
            let pattern_cn = Regex::new(r"（\s*）|（）").unwrap();
            if pattern_en.is_match(&question) {
                let answer_en = format!("({})", user_input);
                let vec = pattern_en
                    .split(&question)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                let question = Line::from(vec![
                    Span::from(format!("{}:: {}", q_index + 1, vec[0].clone())),
                    Span::styled(answer_en, self.select_style(state, user_input.as_str())),
                    Span::from(vec[1].clone()),
                ]);
                lines.push(question);
                return Text::from(lines);
            }
            if pattern_cn.is_match(&question) {
                let answer_cn = format!("（{}）", user_input);
                let vec = pattern_cn
                    .split(&question)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                let question = Line::from(vec![
                    Span::from(format!("{}:: {}", q_index + 1, vec[0].clone())),
                    Span::styled(answer_cn, self.select_style(state, user_input.as_str())),
                    Span::from(vec[1].clone()),
                ]);
                lines.push(question);
                return Text::from(lines);
            }
        }
        lines.push(Line::from(format!("{}: {question}", q_index + 1)));
        Text::from(lines)
    }

    fn user_input(&self) -> Option<String> {
        self.user_input.clone()
    }

    fn answer(&self) -> String {
        self.answer.clone()
    }

    fn score(&self) -> u16 {
        self.score
    }

    fn answered(&self) -> bool {
        self.user_input.is_some()
    }
}

impl Judge {
    fn select_style(&self, state: State, user_input: &str) -> Style {
        match state {
            State::End => {
                if user_input == self.answer.as_str() {
                    return *RIGHT_STYLE;
                }
                *WRONG_STYLE
            }
            State::Ing => *SELECT_STYLE,
        }
    }
}

impl Question for FillIn {
    // TODO 如何高亮显示用户输入文本，以及显示正确与错误的样式
    fn convert_text(&self, state: State, q_index: usize) -> Text<'_> {
        let question = self.question.clone();
        let pattern_en = Regex::new(r"\(\s*\)|\(\)").unwrap();
        let pattern_cn = Regex::new(r"（\s*）|（）").unwrap();
        if pattern_en.is_match(&question) {
            let mut spans = self
                .items
                .iter()
                .map(|item| match item.user_input.clone() {
                    None => Span::styled("()", *DEFAULT_STYLE),
                    Some(user_input) => {
                        let answer_en = format!("({})", user_input);
                        Span::styled(answer_en, *SELECT_STYLE)
                    }
                })
                .collect::<Vec<Span>>();
            spans.push(Span::default());
            let vec = pattern_en
                .split(&question)
                .enumerate()
                .map(|(i, s)| {
                    if i == 1 {
                        Span::styled(format!("{}:: {}", q_index + 1, s), *DEFAULT_STYLE)
                    } else {
                        Span::styled(s.to_string(), *DEFAULT_STYLE)
                    }
                })
                .collect::<Vec<Span>>();
            let spans: Vec<Span> = vec
                .into_iter()
                .zip(spans.into_iter())
                .map(|(s1, s2)| vec![s1, s2])
                .flatten()
                .collect();
            return Text::from(Line::from(spans));
        }
        if pattern_cn.is_match(&question) {
            let mut spans = self
                .items
                .iter()
                .map(|item| match item.user_input.clone() {
                    None => Span::styled("（）", *DEFAULT_STYLE),
                    Some(user_input) => {
                        let answer_en = format!("（{}）", user_input);
                        Span::styled(answer_en, *SELECT_STYLE)
                    }
                })
                .collect::<Vec<Span>>();
            spans.push(Span::default());
            let vec = pattern_cn
                .split(&question)
                .enumerate()
                .map(|(i, s)| {
                    if i == 1 {
                        Span::styled(format!("{}:: {}", q_index + 1, s), *DEFAULT_STYLE)
                    } else {
                        Span::styled(s.to_string(), *DEFAULT_STYLE)
                    }
                })
                .collect::<Vec<Span>>();
            let spans: Vec<Span> = vec
                .into_iter()
                .zip(spans.into_iter())
                .map(|(s1, s2)| vec![s1, s2])
                .flatten()
                .collect();
            return Text::from(Line::from(spans));
        }
        let mut lines = vec![];
        let question = self.question.clone();
        lines.push(Line::from(format!("{}: {question}", q_index + 1)));
        Text::from(lines)
    }

    fn cal_score(&self) -> u16 {
        self.items
            .iter()
            .map(|item| match item.user_input.clone() {
                Some(user_input) if user_input == item.answer => item.score,
                _ => 0,
            })
            .sum()
    }

    fn user_input(&self) -> Option<String> {
        unimplemented!("无需为填空题实现该方法")
    }

    fn answer(&self) -> String {
        unimplemented!("无需为填空题实现该方法")
    }

    fn score(&self) -> u16 {
        unimplemented!("无需为填空题实现该方法")
    }
    fn answered(&self) -> bool {
        self.items.iter().all(|item| item.user_input.is_some())
    }
}

impl FillIn {
    fn user_input(&self) -> Vec<Option<String>> {
        self.items
            .iter()
            .map(|item| item.user_input.clone())
            .collect()
    }

    // fn select_style(&self, state: State, user_input: &str) -> Style {
    // match state {
    //     State::End => {
    //         if user_input == self.answer.as_str() {
    //             return *RIGHT_STYLE;
    //         }
    //         *WRONG_STYLE
    //     }
    //     State::Ing => *SELECT_STYLE,
    // }
    // }
}

fn to_idx(answer: &str) -> Option<usize> {
    match answer {
        "A" | "a" => Some(0),
        "B" | "b" => Some(1),
        "C" | "c" => Some(2),
        "D" | "d" => Some(3),
        "E" | "e" => Some(4),
        "F" | "f" => Some(5),
        "G" | "g" => Some(6),
        "H" | "h" => Some(7),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use crate::components::examination::question::{MultiSelect, Question};
    use regex::Regex;

    #[test]
    fn test_cal_score() {
        let multi_select = MultiSelect {
            question: "question".to_string(),
            options: vec!["A".to_string(), "B".to_string()],
            answer: "AB".to_string(),
            user_input: Some("ba".to_string()),
            score: 1,
        };
        let score = multi_select.cal_score();
        assert_eq!(score, 1);
    }

    #[test]
    fn test_regex() {
        let pattern = Regex::new(r"\(\s*\)|\(\)|（\s*）|（）").unwrap();
        let question = "太阳东升西落，对吗？（ ）";
        let vec = pattern.split(question).collect::<Vec<&str>>();
        assert_eq!(vec, vec!["太阳东升西落，对吗？", ""])
    }
}
