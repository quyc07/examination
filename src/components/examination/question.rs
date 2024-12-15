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
    pub user_input: String,
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

impl Default for Questions<SelectQuestion> {
    fn default() -> Self {
        Questions(load_questions())
    }
}
static DEFAULT_STYLE: LazyLock<Style, fn() -> Style> = LazyLock::new(Style::default);
static SELECT_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Yellow));
static RIGHT_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Green));
static WRONG_STYLE: LazyLock<Style, fn() -> Style> =
    LazyLock::new(|| Style::default().fg(Color::Red));

// static  DEFAULT_STYLE: Style = Style::default();
// static  SELECT_STYLE: Style = Style::default().bg(Color::Yellow);
// static  RIGHT_STYLE: Style = Style::default().bg(Color::Green);
// static  WRONG_STYLE: Style = Style::default().bg(Color::Red);

impl SelectQuestion {
    // TODO 需要考虑题目和选项长度，是否需要折行
    fn cal_total_length(&self) -> u16 {
        (self.options.len() + 1) as u16
    }
    fn question_length(&self) -> u16 {
        1u16
    }

    fn option_length(_option: &str) -> u16 {
        1u16
    }

    pub(crate) fn convert_lines<'a>(&self, state: &State, i: usize) -> Text<'a> {
        let mut lines = vec![];
        let mut question = self.question.clone();
        if !self.user_input.is_empty() {
            let answer = format!("（{}）", self.user_input);
            question = question.replace("（ ）", answer.as_str());
        }
        lines.push(Line::from(format!("{}: {question}", i + 1)));
        for (i, option) in self.options.iter().enumerate() {
            let user_input_idx = answer_to_idx(self.user_input.as_str());
            let answer_idx = answer_to_idx(self.answer.as_str());
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

fn answer_to_idx(answer: &str) -> Option<usize> {
    if answer.is_empty() {
        return None;
    }
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

impl From<&SelectQuestion> for Text<'_> {
    fn from(q: &SelectQuestion) -> Self {
        let mut text = vec![];
        let mut question = q.question.clone();
        if !q.user_input.is_empty() {
            let answer = format!("（{}）", q.user_input);
            question = question.replace("（ ）", answer.as_str());
        }
        text.push(Line::from(question));
        for option in &q.options {
            text.push(Line::from(option.clone()));
        }
        Text::from(text)
    }
}

fn load_questions() -> Vec<SelectQuestion> {
    let questions = vec![
        SelectQuestion {
            question: "北京奥运会于（ ）年举办".to_string(),
            options: vec![
                "A：1998".to_string(),
                "B: 2008".to_string(),
                "C: 2018".to_string(),
                "D: 2020".to_string(),
            ],
            answer: "B".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京冬奥会于（ ）年举办".to_string(),
            options: vec![
                "A：1992".to_string(),
                "B: 2002".to_string(),
                "C: 2012".to_string(),
                "D: 2022".to_string(),
            ],
            answer: "D".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京奥运会于（ ）年举办".to_string(),
            options: vec![
                "A：1998".to_string(),
                "B: 2008".to_string(),
                "C: 2018".to_string(),
                "D: 2020".to_string(),
            ],
            answer: "B".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京冬奥会于（ ）年举办".to_string(),
            options: vec![
                "A：1992".to_string(),
                "B: 2002".to_string(),
                "C: 2012".to_string(),
                "D: 2022".to_string(),
            ],
            answer: "D".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京奥运会于（ ）年举办".to_string(),
            options: vec![
                "A：1998".to_string(),
                "B: 2008".to_string(),
                "C: 2018".to_string(),
                "D: 2020".to_string(),
            ],
            answer: "B".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京冬奥会于（ ）年举办".to_string(),
            options: vec![
                "A：1992".to_string(),
                "B: 2002".to_string(),
                "C: 2012".to_string(),
                "D: 2022".to_string(),
            ],
            answer: "D".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京奥运会于（ ）年举办".to_string(),
            options: vec![
                "A：1998".to_string(),
                "B: 2008".to_string(),
                "C: 2018".to_string(),
                "D: 2020".to_string(),
            ],
            answer: "B".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京冬奥会于（ ）年举办".to_string(),
            options: vec![
                "A：1992".to_string(),
                "B: 2002".to_string(),
                "C: 2012".to_string(),
                "D: 2022".to_string(),
            ],
            answer: "D".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京奥运会于（ ）年举办".to_string(),
            options: vec![
                "A：1998".to_string(),
                "B: 2008".to_string(),
                "C: 2018".to_string(),
                "D: 2020".to_string(),
            ],
            answer: "B".to_string(),
            user_input: "".to_string(),
        },
        SelectQuestion {
            question: "北京冬奥会于（ ）年举办".to_string(),
            options: vec![
                "A：1992".to_string(),
                "B: 2002".to_string(),
                "C: 2012".to_string(),
                "D: 2022".to_string(),
            ],
            answer: "D".to_string(),
            user_input: "".to_string(),
        },
    ];
    questions
}

#[cfg(test)]
mod test {
    use crate::components::examination::question::load_questions;

    #[test]
    fn test() {
        let questions = load_questions();
        let string = serde_json::to_string(&questions).unwrap();
        println!("{}", string);
    }
}
