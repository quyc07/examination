use crate::components::examination::{QuestionEnum, State};
use crate::config::Config;
use ratatui::prelude::{Line, Text};
use ratatui::style::{Color, Style};
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
            Some(user_idx) => match state {
                State::Ing => {
                    if user_idx.contains(&i) {
                        *SELECT_STYLE
                    } else {
                        *DEFAULT_STYLE
                    }
                }
                State::End => {
                    if answer_idx.contains(&i) && user_idx.contains(&i) {
                        *RIGHT_STYLE
                    } else if user_idx.contains(&i) {
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
                let user_input_set = user_input.chars().collect::<HashSet<_>>();
                let answer_set = self.answer().chars().collect::<HashSet<_>>();
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

    fn set_user_input(&mut self, user_input: Option<String>);
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
        }
    }

    pub fn user_input(&self) -> Option<String> {
        match self {
            QuestionEnum::SingleSelect(q) => q.user_input(),
            QuestionEnum::MultiSelect(q) => q.user_input(),
            QuestionEnum::Judge(q) => q.user_input(),
        }
    }

    pub fn set_user_input(&mut self, user_input: Option<String>) {
        match self {
            QuestionEnum::SingleSelect(q) => q.set_user_input(user_input),
            QuestionEnum::MultiSelect(q) => q.set_user_input(user_input),
            QuestionEnum::Judge(q) => q.set_user_input(user_input),
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

    fn set_user_input(&mut self, user_input: Option<String>) {
        self.user_input = user_input;
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

    fn set_user_input(&mut self, user_input: Option<String>) {
        self.user_input = user_input;
    }
}

impl Question for Judge {
    fn convert_text(&self, state: State, q_index: usize) -> Text<'_> {
        let mut lines = vec![];
        let mut question = self.question.clone();
        if let Some(user_input) = &self.user_input {
            let answer = format!("（{}）", user_input);
            question = question.replace("（ ）", answer.as_str());
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

    fn set_user_input(&mut self, user_input: Option<String>) {
        self.user_input = user_input;
    }
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
    use std::collections::HashSet;

    #[test]
    fn test_cal_score() {
        let multi_select = MultiSelect {
            question: "question".to_string(),
            options: vec!["A".to_string(), "B".to_string()],
            answer: "AB".to_string(),
            user_input: Some("ab".to_string()),
            score: 1,
        };
        let score = multi_select.cal_score();
        assert_eq!(score, 1);
        let vec1 = [1, 2, 3];
        let vec2 = [2, 1, 3];
        let hash_set1 = vec1.iter().collect::<HashSet<_>>();
        let hash_set2 = vec2.iter().collect::<HashSet<_>>();
        assert!(hash_set1.eq(&hash_set2));
    }
}
