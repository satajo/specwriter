pub mod integrator;
pub mod ui;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.should_quit = true;
            }
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.submit();
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.insert_newline();
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                self.backspace();
            }
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {
                self.delete();
            }
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                self.move_left();
            }
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                self.move_right();
            }
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => {
                self.move_home();
            }
            KeyEvent {
                code: KeyCode::End,
                ..
            } => {
                self.move_end();
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => {
                self.insert_char(c);
            }
            _ => {}
        }
    }
}

/// Test-friendly runner that drives the full TUI through a TestBackend.
/// Every key event goes through handle_key, every render goes through ui::draw,
/// and the screen buffer is inspectable as text.
pub struct AppRunner {
    pub app: App,
    pub terminal: Terminal<TestBackend>,
    pub ui_rx: mpsc::UnboundedReceiver<IntegratorMessage>,
}

impl std::fmt::Debug for AppRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppRunner")
            .field("app", &self.app)
            .field("terminal", &"<Terminal<TestBackend>>")
            .finish()
    }
}

impl AppRunner {
    pub fn new(config: IntegratorConfig, width: u16, height: u16) -> Self {
        let (app, ui_rx) = App::with_config(config);
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).unwrap();
        Self {
            app,
            terminal,
            ui_rx,
        }
    }

    /// Render the UI and return the screen content as a string.
    pub fn render(&mut self) -> String {
        self.terminal
            .draw(|f| ui::draw(f, &self.app))
            .unwrap();
        let buf = self.terminal.backend().buffer().clone();
        let mut lines = Vec::new();
        for y in 0..buf.area.height {
            let mut line = String::new();
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                line.push_str(cell.symbol());
            }
            lines.push(line.trim_end().to_string());
        }
        // Drop trailing empty lines
        while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.pop();
        }
        lines.join("\n")
    }

    /// Send a key event through the full handle_key path, then drain integrator messages.
    pub fn send_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let key = KeyEvent::new(code, modifiers);
        self.app.handle_key(key);
        self.drain_pending();
    }

    /// Type a string character by character through handle_key.
    pub fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.send_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
    }

    /// Press Ctrl+S (submit) through handle_key.
    pub fn submit(&mut self) {
        self.send_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
    }

    /// Drain any pending integrator messages into the app (non-blocking).
    pub fn drain_pending(&mut self) {
        while let Ok(msg) = self.ui_rx.try_recv() {
            self.app.update_from_integrator(msg);
        }
    }

    /// Wait for an integration cycle to complete (status contains "complete" or "Error").
    pub async fn wait_for_integration(&mut self) {
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                self.ui_rx.recv(),
            )
            .await
            {
                Ok(Some(msg)) => {
                    let done = matches!(
                        &msg,
                        IntegratorMessage::StatusUpdate(s) if s.contains("complete") || s.contains("Error")
                    );
                    self.app.update_from_integrator(msg);
                    if done {
                        // Drain anything else that arrived
                        while let Ok(msg) = self.ui_rx.try_recv() {
                            self.app.update_from_integrator(msg);
                        }
                        return;
                    }
                }
                Ok(None) => return,
                Err(_) => continue,
            }
        }
        panic!("Timed out waiting for integration to complete");
    }

    /// Wait until all pending integrations are done (no new completions for 500ms).
    pub async fn wait_until_idle(&mut self) {
        loop {
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(500),
                self.ui_rx.recv(),
            )
            .await
            {
                Ok(Some(msg)) => {
                    self.app.update_from_integrator(msg);
                }
                _ => return, // timeout (idle) or channel closed
            }
        }
    }

    /// Check whether a string appears anywhere on the rendered screen.
    pub fn screen_contains(&mut self, needle: &str) -> bool {
        let screen = self.render();
        screen.contains(needle)
    }
}
