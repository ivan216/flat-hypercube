use crossterm::event::KeyCode;

pub const OUTPUT_HISTORY_LIMIT: usize = 500;
pub const OUTPUT_PANEL_MAX_HEIGHT: u16 = 10;
pub const OUTPUT_PANEL_TOGGLE_KEY: char = '?';

#[derive(Debug, Clone, Default)]
pub struct CommandInput {
    pub buffer: String,
    pub history: Vec<String>,
    pub history_cursor: Option<usize>,
}

impl CommandInput {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.history_cursor = None;
    }

    pub fn push_char(&mut self, ch: char) {
        self.buffer.push(ch);
        self.history_cursor = None;
    }

    pub fn backspace(&mut self) {
        self.buffer.pop();
        self.history_cursor = None;
    }

    pub fn submit(&mut self) -> Option<String> {
        let command = self.buffer.trim().to_string();
        self.clear();
        if command.is_empty() {
            return None;
        }

        self.history.push(command.clone());
        Some(command)
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let next = match self.history_cursor {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_cursor = Some(next);
        self.buffer = self.history[next].clone();
    }

    pub fn history_next(&mut self) {
        let Some(cursor) = self.history_cursor else {
            return;
        };

        if cursor + 1 >= self.history.len() {
            self.history_cursor = None;
            self.buffer.clear();
        } else {
            let next = cursor + 1;
            self.history_cursor = Some(next);
            self.buffer = self.history[next].clone();
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, output: &mut CommandOutput) -> CommandKeyAction {
        match code {
            KeyCode::Esc => {
                self.clear();
                CommandKeyAction::Cancel
            }
            KeyCode::Enter => self
                .submit()
                .map(CommandKeyAction::Submit)
                .unwrap_or(CommandKeyAction::None),
            KeyCode::Backspace => {
                self.backspace();
                CommandKeyAction::None
            }
            KeyCode::Char(c) => {
                self.push_char(c);
                CommandKeyAction::None
            }
            KeyCode::Up => {
                self.history_prev();
                CommandKeyAction::None
            }
            KeyCode::Down => {
                self.history_next();
                CommandKeyAction::None
            }
            KeyCode::PageUp if output.open => {
                output.page_up();
                CommandKeyAction::None
            }
            KeyCode::PageDown if output.open => {
                output.page_down();
                CommandKeyAction::None
            }
            _ => CommandKeyAction::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKeyAction {
    None,
    Cancel,
    Submit(String),
}

#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    pub lines: Vec<String>,
    pub scroll: usize,
    pub open: bool,
    pub hint: Option<String>,
}

impl CommandOutput {
    pub fn visible(&self) -> bool {
        self.open && !self.lines.is_empty()
    }

    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        if self.lines.len() > OUTPUT_HISTORY_LIMIT {
            let trim = self.lines.len() - OUTPUT_HISTORY_LIMIT;
            self.lines.drain(0..trim);
        }
        self.scroll = 0;
    }

    pub fn push_lines(&mut self, lines: impl IntoIterator<Item = String>) {
        for line in lines {
            self.push_line(line);
        }
    }

    pub fn push_command_echo(&mut self, command: &str) {
        if command.eq_ignore_ascii_case("panel") {
            return;
        }

        self.push_line(format!("> /{}", command));
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll = 0;
        self.open = false;
        self.hint = None;
    }

    pub fn toggle(&mut self) -> Option<&'static str> {
        if self.lines.is_empty() {
            return Some("output panel is empty");
        }

        self.open = !self.open;
        self.hint = None;
        Some(if self.open {
            "output panel shown"
        } else {
            "output panel hidden"
        })
    }

    pub fn page_up(&mut self) {
        if self.open {
            self.scroll = self.scroll.saturating_add(5);
        }
    }

    pub fn page_down(&mut self) {
        if self.open {
            self.scroll = self.scroll.saturating_sub(5);
        }
    }

    pub fn clamp_scroll(&mut self, visible_height: u16) {
        if visible_height > 0 {
            let max_scroll = self.lines.len().saturating_sub(visible_height as usize);
            self.scroll = self.scroll.min(max_scroll);
        } else {
            self.scroll = 0;
        }
    }

    pub fn set_closed_hint_from(&mut self, start: usize) {
        if !self.open {
            self.hint = self.lines.get(start).cloned();
        }
    }
}

pub fn command_help_lines() -> Vec<String> {
    vec![
        "commands:".to_string(),
        "/help - show this list".to_string(),
        "/clear - clear the output panel".to_string(),
        "/panel or ? - fold or unfold the output panel".to_string(),
        "/status - show puzzle and input state".to_string(),
        "/moves - show the last 20 moves".to_string(),
        "/save - save the current session".to_string(),
        "/scramble - scramble the puzzle".to_string(),
        "/reset - reset the puzzle".to_string(),
        "/rev start|stop|unwind|comm - manage reversion blocks".to_string(),
    ]
}
