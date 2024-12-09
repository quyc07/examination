use std::ops::Deref;
use ratatui::prelude::{Line, Text};
use crate::components::user_input::UserInput;

pub struct SelectQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub answer: String,
}

pub struct Questions<T>(pub(crate) Vec<T>);

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
}

impl<'a> From<&SelectQuestion> for Text<'a> {
    fn from(question: &SelectQuestion) -> Self {
        let mut text = vec![];
        text.push(Line::from(question.question.clone()));
        for option in &question.options {
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
        },
    ];
    questions
}