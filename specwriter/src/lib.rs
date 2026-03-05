pub mod integrator;
pub mod ui;

use tokio::sync::mpsc;

use integrator::{IntegratorConfig, IntegratorHandle, IntegratorMessage};

#[derive(Debug)]
pub struct App {
    pub input: String,
    pub cursor_pos: usize,
    pub questions: Vec<String>,
    pub status: String,
    pub integrator: IntegratorHandle,
    pub should_quit: bool,
}

impl App {
    pub fn new(integrator: IntegratorHandle) -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            questions: Vec::new(),
            status: "Ready. Type your requirements and press Ctrl+S to submit.".into(),
            integrator,
            should_quit: false,
        }
    }

    pub fn with_default_integrator() -> (Self, mpsc::UnboundedReceiver<IntegratorMessage>) {
        let (ui_tx, ui_rx) = mpsc::unbounded_channel();
        let integrator = IntegratorHandle::new(ui_tx, IntegratorConfig::default());
        (Self::new(integrator), ui_rx)
    }

    pub fn with_config(
        config: IntegratorConfig,
    ) -> (Self, mpsc::UnboundedReceiver<IntegratorMessage>) {
        let (ui_tx, ui_rx) = mpsc::unbounded_channel();
        let integrator = IntegratorHandle::new(ui_tx, config);
        (Self::new(integrator), ui_rx)
    }

    pub fn submit(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.integrator.send(text);
        self.input.clear();
        self.cursor_pos = 0;
        self.status = "Submitted. Integrating...".into();
    }

    pub fn update_from_integrator(&mut self, msg: IntegratorMessage) {
        match msg {
            IntegratorMessage::QuestionsUpdated(q) => self.questions = q,
            IntegratorMessage::StatusUpdate(s) => self.status = s,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn insert_newline(&mut self) {
        self.input.insert(self.cursor_pos, '\n');
        self.cursor_pos += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input.replace_range(prev..self.cursor_pos, "");
            self.cursor_pos = prev;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_pos < self.input.len() {
            let next = self.input[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.input.len());
            self.input.replace_range(self.cursor_pos..next, "");
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = self.input[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.cursor_pos = self.input[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.input.len());
        }
    }

    pub fn move_home(&mut self) {
        let before = &self.input[..self.cursor_pos];
        self.cursor_pos = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    }

    pub fn move_end(&mut self) {
        let after = &self.input[self.cursor_pos..];
        self.cursor_pos += after.find('\n').unwrap_or(after.len());
    }
}
