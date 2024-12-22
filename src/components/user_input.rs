use crate::action::Action;
use crate::app::{Mode, ModeHolder};
use crate::components::area_util::{centered_rect, sub_area};
use crate::components::examination::QuestionEnum;
use crate::components::Component;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

// TODO 需要实现widget，实现前台悬浮展示功能
pub struct UserInput {
    /// Current value of the input box
    input: Vec<Option<String>>,
    /// 当前正在输入的字符串索引
    current_input_idx: Option<usize>,
    /// 加载组件时带入的用户输入
    question: Option<QuestionEnum>,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// 问题请求
    question_rx: UnboundedReceiver<QuestionEnum>,
    /// 答案
    answer_tx: UnboundedSender<QuestionEnum>,
    /// 全局状态
    state_holder: Arc<Mutex<ModeHolder>>,
    /// 输入类型
    input_type: InputType,
}

#[derive(Default)]
enum InputType {
    #[default]
    Fill,
    Judge,
}

impl Component for UserInput {
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        if self.get_state() == Mode::Input {
            match self.input_type {
                InputType::Fill => match key.code {
                    KeyCode::Tab => self.move_cursor_next(),
                    KeyCode::Enter => self.submit_message(),
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Esc => self.close(),
                    _ => {}
                },
                InputType::Judge => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.input = vec![Some("Yes".to_string())];
                        self.submit_message()
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.input = vec![Some("No".to_string())];
                        self.submit_message()
                    }
                    _ => {}
                },
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        match self.get_state() {
            Mode::Examination => {
                if let Ok(q) = self.question_rx.try_recv() {
                    match q {
                        QuestionEnum::SingleSelect(_) => self.input_type = InputType::Fill,
                        QuestionEnum::MultiSelect(_) => self.input_type = InputType::Fill,
                        QuestionEnum::Judge(_) => self.input_type = InputType::Judge,
                        QuestionEnum::FillIn(_) => self.input_type = InputType::Fill,
                    }
                    self.input = q.user_input();
                    self.current_input_idx = Some(0);
                    self.question = Some(q);
                    self.state_holder.lock().unwrap().set_mode(Mode::Input);
                }
            }
            Mode::Input => match self.input_type {
                InputType::Fill => {
                    self.draw_fill(frame, area);
                }
                InputType::Judge => {
                    Self::draw_judge(frame, area);
                }
            },
            _ => {}
        }
        Ok(())
    }
}

impl UserInput {
    pub fn new(
        question_rx: UnboundedReceiver<QuestionEnum>,
        answer_tx: UnboundedSender<QuestionEnum>,
        state_holder: Arc<Mutex<ModeHolder>>,
    ) -> Self {
        Self {
            input: vec![None],
            current_input_idx: None,
            question: None,
            character_index: 0,
            question_rx,
            answer_tx,
            state_holder,
            input_type: InputType::default(),
        }
    }

    fn get_state(&self) -> Mode {
        self.state_holder.lock().unwrap().mode
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        let mut input = self.current_input();
        input.insert(index, new_char);
        self.set_current_input(input);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.current_input()
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.current_input().len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let input = self.current_input();
            let before_char_to_delete = input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            let input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.set_current_input(input);
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.current_input().chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn current_input(&self) -> String {
        self.input[self.current_input_idx.unwrap()]
            .clone()
            .unwrap_or_default()
    }

    fn set_current_input(&mut self, input: String) {
        self.input[self.current_input_idx.unwrap()] = Some(input);
    }

    fn move_cursor_next(&mut self) {
        if self.question.clone().unwrap().input_size() == 1 {
            return;
        }
        match self.current_input_idx {
            None => {
                panic!("未设置正确文本框索引！")
            }
            Some(idx) => {
                let next_idx = idx + 1;
                if next_idx < self.question.clone().unwrap().input_size() {
                    self.current_input_idx = Some(next_idx);
                } else {
                    self.current_input_idx = Some(0);
                }
            }
        }
        self.reset_cursor();
    }

    fn submit_message(&mut self) {
        let mut question = self.question.take().unwrap();
        question.set_user_input(self.input.clone());
        self.answer_tx.send(question).unwrap();
        self.reset()
    }

    fn reset(&mut self) {
        self.input.clear();
        self.question.take();
        self.reset_cursor();
    }

    fn close(&mut self) {
        self.answer_tx.send(self.question.take().unwrap()).unwrap();
        self.reset()
    }

    fn draw_judge(frame: &mut Frame, area: Rect) {
        let area = centered_rect(50, 20, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Press Esc to stop exist, Press Y to answer Yes, Press N to answer No.")
            .title_alignment(Alignment::Center);
        frame.render_widget(block, area);
        let [_, yes_area, no_area, _] = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .areas(area);
        let [_, yes_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(yes_area);
        let [_, no_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(no_area);
        let yes = Paragraph::new("Yes(Y)")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .style(Style::default().fg(Color::Yellow))
                    .borders(Borders::ALL),
            );
        frame.render_widget(yes, yes_area);
        let no = Paragraph::new("No(N)")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .style(Style::default().fg(Color::Yellow))
                    .borders(Borders::ALL),
            );
        frame.render_widget(no, no_area);
    }

    fn draw_fill(&mut self, frame: &mut Frame, area: Rect) {
        let area = centered_rect(70, 30, area);
        let input_size = self.question.clone().unwrap().input_size();
        let title = if input_size == 1 {
            "Press Esc to stop exist, Press Enter to submit answer."
        } else {
            "Press Esc to stop exist, Press Tab to switch to next editor, Press Enter to submit answer."
        };
        frame.render_widget(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center),
            area,
        );
        let [_blank, mut area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .areas(area);
        let mut input_areas = vec![];
        for i in 0..input_size {
            let (input_area, rest) = sub_area(Constraint::Length(3), Direction::Vertical, area);
            let input_area = centered_rect(50, 100, input_area);
            let input = Paragraph::new(self.input[i].clone().unwrap_or_default())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(input, input_area);
            area = rest;
            input_areas.push(input_area);
        }
        self.set_cursor_position(frame, input_areas[self.current_input_idx.unwrap()]);
    }

    fn set_cursor_position(&mut self, frame: &mut Frame, input_area: Rect) {
        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            input_area.x + self.character_index as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        ));
    }
}

#[cfg(test)]
mod test {
    use crate::app::ModeHolder;
    use crate::components::user_input::UserInput;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    #[test]
    fn test_byte_index() {
        let (_, question_rx) = mpsc::unbounded_channel();
        let (answer_tx, _) = mpsc::unbounded_channel();
        let mut input = UserInput::new(
            question_rx,
            answer_tx,
            Arc::new(Mutex::new(ModeHolder::default())),
        );
        input.current_input_idx = Some(0);
        assert_eq!(input.byte_index(), 0)
    }
}
