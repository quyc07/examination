use crate::action::Action;
use crate::app::{Mode, ModeHolderLock};
use crate::components::Component;
use crate::components::area_util::centered_rect;
use crate::components::examination::QuestionEnum;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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
    mode_holder: ModeHolderLock,
    /// 输入类型
    input_type: InputType,
    /// 输入框光标位置
    cursor_position: Option<Position>,
}

#[derive(Default)]
enum InputType {
    #[default]
    Fill,
    Judge,
}

impl Widget for &mut UserInput {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.mode_holder.get_mode() {
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
                    self.mode_holder.set_mode(Mode::Input);
                }
            }

            Mode::Input => match self.input_type {
                InputType::Fill => {
                    self.draw_fill(area, buf);
                }
                InputType::Judge => {
                    self.draw_judge(area, buf);
                }
            },
            _ => {}
        }
    }
}

impl Component for UserInput {
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        if self.mode_holder.get_mode() == Mode::Input {
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
                    KeyCode::Esc => self.close(),
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
        frame.render_widget(&mut *self, area);
        if let Some(position) = self.cursor_position {
            frame.set_cursor_position(position)
        }
        Ok(())
    }
}

impl UserInput {
    pub fn new(
        question_rx: UnboundedReceiver<QuestionEnum>,
        answer_tx: UnboundedSender<QuestionEnum>,
        state_holder: ModeHolderLock,
    ) -> Self {
        Self {
            input: vec![None],
            current_input_idx: None,
            question: None,
            character_index: 0,
            question_rx,
            answer_tx,
            mode_holder: state_holder,
            input_type: InputType::default(),
            cursor_position: None,
        }
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
        self.cursor_position = None;
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

    fn draw_judge(&self, area: Rect, buf: &mut Buffer) {
        let area = centered_rect(50, 20, area);
        Clear.render(area, buf);
        Block::default()
            .borders(Borders::ALL)
            .title("Esc to exist, Y to answer Yes, N to answer No.")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray).bg(Color::DarkGray))
            .render(area, buf);
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
        Paragraph::new("Yes(Y)")
            .style(Style::default())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .style(Style::default())
                    .borders(Borders::ALL),
            )
            .render(yes_area, buf);
        Paragraph::new("No(N)")
            .style(Style::default())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .style(Style::default())
                    .borders(Borders::ALL),
            )
            .render(no_area, buf);
    }

    fn draw_fill(&mut self, area: Rect, buf: &mut Buffer) {
        let input_size = self.question.clone().unwrap().input_size();
        let area = centered_rect(50, 100, area);
        let high = Self::cal_high(input_size, area);
        let [_, area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(high),
                Constraint::Fill(1),
            ])
            .areas(area);
        Clear.render(area, buf);

        let title = if input_size == 1 {
            "Esc to exist, Enter to submit answer."
        } else {
            "Esc to exist, Tab to switch, Enter to submit answer."
        };
        Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray).bg(Color::DarkGray))
            .render(area, buf);
        let [_, area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length((input_size * 3) as u16),
                Constraint::Fill(1),
            ])
            .areas(area);
        let constraints = (0..input_size)
            .map(|_| Constraint::Length(3))
            .collect::<Vec<Constraint>>();
        let rects = Layout::vertical(constraints).split(area);

        let mut input_areas = vec![];
        for i in 0..rects.len() {
            let area = centered_rect(80, 100, rects[i]);
            Paragraph::new(self.input[i].clone().unwrap_or_default())
                .style(Style::default())
                .block(Block::default().borders(Borders::ALL))
                .render(area, buf);
            input_areas.push(area);
        }
        self.set_cursor_position(input_areas[self.current_input_idx.unwrap()]);
    }

    fn cal_high(input_size: usize, area: Rect) -> u16 {
        let total_length = (input_size * 3 + 2) as u16;
        if area.height / 5 > total_length {
            area.height / 5
        } else {
            total_length
        }
    }

    fn set_cursor_position(&mut self, input_area: Rect) {
        self.cursor_position = Some(Position::new(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            input_area.x + self.character_index as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::app::{ModeHolder, ModeHolderLock};
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
            ModeHolderLock(Arc::new(Mutex::new(ModeHolder::default()))),
        );
        input.current_input_idx = Some(0);
        assert_eq!(input.byte_index(), 0)
    }
}
