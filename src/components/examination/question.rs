use crate::components::examination::State;
use crate::config::Config;
use ratatui::prelude::{Line, Text};
use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::sync::LazyLock;

#[derive(Serialize, Deserialize)]
pub struct SelectQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
    pub user_input: Option<String>,
    pub score: u16,
}

pub struct Questions<T>(pub(crate) Vec<T>);

impl Questions<SelectQuestion> {
    pub(crate) fn load(config: Config) -> Questions<SelectQuestion> {
        let mut questions = String::new();
        File::open(config.config.data_dir.join("question.json"))
            .unwrap()
            .read_to_string(&mut questions)
            .expect("Fail to load question!");
        let select_question =
            serde_json::from_slice::<Vec<SelectQuestion>>(questions.as_ref()).unwrap();
        Questions(select_question)
    }
}

impl<T> Deref for Questions<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static DEFAULT_STYLE: LazyLock<Style, fn() -> Style> = LazyLock::new(Style::default);
static SELECT_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Yellow));
static RIGHT_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Green));
static WRONG_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Red));

impl SelectQuestion {
    pub(crate) fn convert_lines<'a>(&self, state: &State, i: usize) -> Text<'a> {
        let mut lines = vec![];
        let mut question = self.question.clone();
        if let Some(user_input) = &self.user_input {
            let answer = format!("（{}）", user_input);
            question = question.replace("（ ）", answer.as_str());
        }
        lines.push(Line::from(format!("{}: {question}", i + 1)));
        for (i, option) in self.options.iter().enumerate() {
            let user_input_idx = answer_to_idx(self.user_input.clone());
            let answer_idx = answer_to_idx(Some(self.answer.clone()));
            let style = Self::check_style(state, i, user_input_idx, answer_idx.unwrap());
            lines.push(Line::from(format!("  {option}")).style(style));
        }
        Text::from(lines)
    }

    fn check_style(
        state: &State,
        i: usize,
        user_input_idx: Option<usize>,
        answer_idx: usize,
    ) -> Style {
        match user_input_idx {
            None => *DEFAULT_STYLE,
            Some(user_idx) => match state {
                State::Ing => {
                    if i == user_idx {
                        *SELECT_STYLE
                    } else {
                        *DEFAULT_STYLE
                    }
                }
                State::End => {
                    if user_idx == answer_idx {
                        if i == user_idx {
                            *RIGHT_STYLE
                        } else {
                            *DEFAULT_STYLE
                        }
                    } else if i == user_idx {
                        *WRONG_STYLE
                    } else if i == answer_idx {
                        *RIGHT_STYLE
                    } else {
                        *DEFAULT_STYLE
                    }
                }
            },
        }
    }
}

fn answer_to_idx(answer: Option<String>) -> Option<usize> {
    answer.and_then(|answer| match answer.as_str() {
        "A" | "a" => Some(0),
        "B" | "b" => Some(1),
        "C" | "c" => Some(2),
        "D" | "d" => Some(3),
        "E" | "e" => Some(4),
        "F" | "f" => Some(5),
        "G" | "g" => Some(6),
        "H" | "h" => Some(7),
        _ => None,
    })
}

impl From<&SelectQuestion> for Text<'_> {
    fn from(q: &SelectQuestion) -> Self {
        let mut text = vec![];
        let mut question = q.question.clone();
        if let Some(user_input) = &q.user_input {
            let answer = format!("（{}）", user_input);
            question = question.replace("（ ）", answer.as_str());
        }
        text.push(Line::from(question));
        for option in &q.options {
            text.push(Line::from(option.clone()));
        }
        Text::from(text)
    }
}
