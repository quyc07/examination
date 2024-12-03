use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use ratatui::style::Styled;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct Examination {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Examination {
    pub fn new() -> Self {
        Self::default()
    }
}

struct SingleSelectQuestion {
    question: String,
    options: Vec<String>,
    answer: String,
}

impl SingleSelectQuestion {
    // TODO 需要考虑题目和选项长度，是否需要折行
    fn cal_total_length(&self) -> u16 {
        (self.options.len() + 1) as u16
    }
    fn question_length(&self) -> u16 {
        1u16
    }

    fn option_length(option: &str) -> u16 {
        1u16
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
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "***考试",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        let area = sub_rect(0, 60, 0, 100, area);
        frame.render_widget(block, area);

        let questions = vec![
            SingleSelectQuestion {
                question: "北京奥运会于（ ）年举办".to_string(),
                options: vec![
                    "A：1998".to_string(),
                    "B: 2008".to_string(),
                    "C: 2018".to_string(),
                    "D: 2020".to_string(),
                ],
                answer: "B".to_string(),
            },
            SingleSelectQuestion {
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

        let mut area = sub_rect(5, 90, 5, 90, area);
        for (index, q) in questions.iter().enumerate() {
            let (total_area, remain) = split_rect(q.cal_total_length(), area);
            area = remain;
            let (q_area, remain) = split_rect(q.question_length(), total_area);
            let mut option_area = remain;
            let mut options = Vec::new();
            for option in &q.options {
                let (op_area, remain) = split_rect(
                    SingleSelectQuestion::option_length(option.as_str()),
                    option_area,
                );
                options.push((option.as_str(), op_area));
                option_area = remain;
            }
            frame.render_widget(
                Paragraph::new(format!("{}. {}", index + 1, q.question.clone()))
                    .style(Style::default().fg(Color::Blue)),
                q_area,
            );
            for (option, area) in options {
                frame.render_widget(
                    Paragraph::new(option).style(Style::default().fg(Color::Blue)),
                    area,
                )
            }
        }
        Ok(())
    }
}

pub(crate) fn total_area(frame: &mut Frame) -> Rect {
    Rect::new(0, 0, frame.area().width * 6 / 10, frame.area().height)
}

fn split_rect(length: u16, r: Rect) -> (Rect, Rect) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(length), Constraint::Fill(1)])
        .split(r);
    (rects[0], rects[1])
}

// ANCHOR: centered_rect
/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
// ANCHOR_END: centered_rect

fn sub_rect(x: u16, width: u16, y: u16, high: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(x),
            Constraint::Percentage(width),
            Constraint::Percentage(100 - x - width),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(y),
            Constraint::Percentage(high),
            Constraint::Percentage(100 - y - high),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
