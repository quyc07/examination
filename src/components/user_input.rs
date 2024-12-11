use crate::action::Action;
use crate::app::{State, StateHolder};
use crate::components::area_util::centered_rect;
use crate::components::Component;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::info;

pub struct UserInput {
    /// Current value of the input box
    pub input: String,
    /// 加载组件时带入的用户输入
    pub origin_input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// 问题请求
    pub question_rx: UnboundedReceiver<String>,
    /// 答案
    pub answer_tx: UnboundedSender<String>,
    /// 全局状态
    state_holder: Arc<Mutex<StateHolder>>,
}

impl Component for UserInput {
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        match self.get_state() {
            State::View => {}
            State::Input => match key.code {
                KeyCode::Enter => self.submit_message(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.close(),
                _ => {}
            },
            State::Submit => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        match self.get_state() {
            State::View => {
                if let Ok(user_input) = self.question_rx.try_recv() {
                    info!("receive {user_input}");
                    self.input = user_input.clone();
                    self.origin_input = user_input;
                    self.state_holder.lock().unwrap().set_state(State::Input);
                }
            }
            State::Input => {
                let area = centered_rect(50, 30, area);
                let vertical = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ]);
                let [help_area, input_area, other] = vertical.areas(area);

                let (msg, style) = (
                    vec!["Press Esc to stop exist, Press Enter to submit answer.".into()],
                    Style::default(),
                );
                let text = Text::from(Line::from(msg)).patch_style(style);
                let help_message = Paragraph::new(text);
                //
                frame.render_widget(help_message, help_area);

                let input = Paragraph::new(self.input.as_str())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(input, input_area);
                frame.set_cursor_position(Position::new(
                    // Draw the cursor at the current position in the input field.
                    // This position is can be controlled via the left and right arrow key
                    area.x + self.character_index as u16 + 1,
                    // Move one line down, from the border to the input line
                    area.y + 2,
                ));
            }
            State::Submit => {}
        }
        Ok(())
    }
}

impl UserInput {
    pub fn new(
        question_rx: UnboundedReceiver<String>,
        answer_tx: UnboundedSender<String>,
        state_holder: Arc<Mutex<StateHolder>>,
    ) -> Self {
        Self {
            input: String::new(),
            origin_input: String::new(),
            character_index: 0,
            question_rx,
            answer_tx,
            state_holder,
        }
    }

    fn get_state(&self) -> State {
        self.state_holder.lock().unwrap().state.clone()
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
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
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
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.answer_tx.send(self.input.clone()).unwrap();
        self.reset()
    }

    fn reset(&mut self) {
        self.input.clear();
        self.origin_input.clear();
        self.reset_cursor();
    }

    fn close(&mut self) {
        self.answer_tx.send(self.origin_input.clone()).unwrap();
        self.reset()
    }
}
