pub mod integrator;
pub mod ui;

pub use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::event::{KeyEvent};
use ratatui::{backend::TestBackend, style::Color, Terminal};
use tokio::sync::mpsc;

use integrator::{IntegratorConfig, IntegratorHandle, IntegratorMessage, Question};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Integrating,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveTab {
    TextInput,
    Questions,
    Spec,
}

#[derive(Debug)]
pub struct AnswerDialog {
    pub question: Question,
    pub input: String,
    pub cursor_pos: usize,
}

#[derive(Debug)]
pub struct App {
    pub input: String,
    pub cursor_pos: usize,
    pub questions: Vec<Question>,
    pub status: String,
    pub state: AppState,
    pub tick: u64,
    pub integrator: IntegratorHandle,
    pub should_quit: bool,
    pub active_tab: ActiveTab,
    pub question_focus: usize,
    pub answer_dialog: Option<AnswerDialog>,
    pub input_scroll: u16,
    pub detail_scroll: u16,
    pub quit_dialog: bool,
    pub spec_content: Option<String>,
    pub spec_scroll: u16,
}

impl App {
    pub fn new(integrator: IntegratorHandle) -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            questions: Vec::new(),
            status: "Idle.".into(),
            state: AppState::Idle,
            tick: 0,
            integrator,
            should_quit: false,
            active_tab: ActiveTab::TextInput,
            question_focus: 0,
            answer_dialog: None,
            input_scroll: 0,
            detail_scroll: 0,
            quit_dialog: false,
            spec_content: None,
            spec_scroll: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub fn with_default_integrator() -> (Self, mpsc::UnboundedReceiver<IntegratorMessage>) {
        let config = IntegratorConfig::default();
        Self::with_config(config)
    }

    pub fn with_config(
        config: IntegratorConfig,
    ) -> (Self, mpsc::UnboundedReceiver<IntegratorMessage>) {
        let (ui_tx, ui_rx) = mpsc::unbounded_channel();
        let spec_path = config.working_dir.join("SPEC.md");
        let initial_questions = integrator::scan_questions(&spec_path);
        let spec_content = if spec_path.exists() {
            Some(std::fs::read_to_string(&spec_path).unwrap_or_default())
        } else {
            None
        };
        let integrator = IntegratorHandle::new(ui_tx, config);
        let mut app = Self::new(integrator);
        app.questions = initial_questions;
        app.spec_content = spec_content;
        (app, ui_rx)
    }

    pub fn submit(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.integrator.send(text);
        self.input.clear();
        self.cursor_pos = 0;
        self.state = AppState::Integrating;
        self.status = "Integrating".into();
    }

    pub fn submit_answer(&mut self) {
        if let Some(ref dialog) = self.answer_dialog {
            let text = dialog.input.trim().to_string();
            if text.is_empty() {
                return;
            }
            let dialog = self.answer_dialog.take().unwrap();
            let message = format!(
                "The answer to question Q{} ({}) is: {}",
                dialog.question.id, dialog.question.text, text
            );
            // Immediately remove the answered question from the UI
            let answered_id = dialog.question.id;
            self.questions.retain(|q| q.id != answered_id);
            if !self.questions.is_empty() && self.question_focus >= self.questions.len() {
                self.question_focus = self.questions.len() - 1;
            }
            self.integrator.send(message);
            self.state = AppState::Integrating;
            self.status = "Integrating".into();
        }
    }

    pub fn update_from_integrator(&mut self, msg: IntegratorMessage) {
        match msg {
            IntegratorMessage::QuestionsUpdated(q) => {
                // Preserve focus by question ID
                let focused_id = self.questions.get(self.question_focus).map(|q| q.id);
                let old_focus = self.question_focus;
                self.questions = q;
                if let Some(id) = focused_id {
                    if let Some(pos) = self.questions.iter().position(|q| q.id == id) {
                        self.question_focus = pos;
                    } else {
                        // Focused question removed — try same index (next), then previous
                        if !self.questions.is_empty() {
                            self.question_focus = old_focus.min(self.questions.len() - 1);
                        } else {
                            self.question_focus = 0;
                        }
                    }
                } else if !self.questions.is_empty() && self.question_focus >= self.questions.len() {
                    self.question_focus = self.questions.len() - 1;
                }
            }
            IntegratorMessage::IntegrationComplete => {
                self.state = AppState::Idle;
                self.status = "Idle.".into();
                self.quit_dialog = false;
                // Refresh spec content
                let spec_path = self.integrator.working_dir().join("SPEC.md");
                self.spec_content = if spec_path.exists() {
                    Some(std::fs::read_to_string(&spec_path).unwrap_or_default())
                } else {
                    None
                };
                self.spec_scroll = 0;
            }
            IntegratorMessage::StatusUpdate(s) => {
                if s.contains("Error") {
                    self.state = AppState::Error;
                }
                self.status = s;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Quit confirmation dialog modal
        if self.quit_dialog {
            match key {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    self.should_quit = true;
                }
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    self.quit_dialog = false;
                }
                _ => {}
            }
            return;
        }

        // Ctrl+C quit handling
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            if self.state == AppState::Integrating {
                self.quit_dialog = true;
            } else {
                self.should_quit = true;
            }
            return;
        }

        // Answer dialog modal
        if self.answer_dialog.is_some() {
            self.handle_answer_dialog_key(key);
            return;
        }

        // Tab switching (global)
        if key.code == KeyCode::Tab && key.modifiers == KeyModifiers::NONE {
            self.active_tab = match self.active_tab {
                ActiveTab::TextInput => ActiveTab::Questions,
                ActiveTab::Questions => ActiveTab::Spec,
                ActiveTab::Spec => ActiveTab::TextInput,
            };
            return;
        }

        match self.active_tab {
            ActiveTab::TextInput => self.handle_text_input_key(key),
            ActiveTab::Questions => self.handle_questions_key(key),
            ActiveTab::Spec => self.handle_spec_key(key),
        }
    }

    fn handle_text_input_key(&mut self, key: KeyEvent) {
        match key {
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

    fn handle_questions_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.question_focus > 0 {
                    self.question_focus -= 1;
                }
            }
            KeyCode::Down => {
                if !self.questions.is_empty() && self.question_focus < self.questions.len() - 1 {
                    self.question_focus += 1;
                }
            }
            KeyCode::Enter => {
                if !self.questions.is_empty() {
                    let q = self.questions[self.question_focus].clone();
                    self.answer_dialog = Some(AnswerDialog {
                        question: q,
                        input: String::new(),
                        cursor_pos: 0,
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_spec_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down => {
                if let Some(ref content) = self.spec_content {
                    let total_lines = content.lines().count().max(1) as u16;
                    if self.spec_scroll < total_lines.saturating_sub(1) {
                        self.spec_scroll += 1;
                    }
                }
            }
            KeyCode::Up => {
                if self.spec_scroll > 0 {
                    self.spec_scroll -= 1;
                }
            }
            _ => {}
        }
    }

    fn handle_answer_dialog_key(&mut self, key: KeyEvent) {
        match key {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.answer_dialog = None;
            }
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.submit_answer();
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    d.input.insert(d.cursor_pos, '\n');
                    d.cursor_pos += 1;
                }
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    if d.cursor_pos > 0 {
                        let prev = d.input[..d.cursor_pos]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        d.input.replace_range(prev..d.cursor_pos, "");
                        d.cursor_pos = prev;
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    if d.cursor_pos < d.input.len() {
                        let next = d.input[d.cursor_pos..]
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| d.cursor_pos + i)
                            .unwrap_or(d.input.len());
                        d.input.replace_range(d.cursor_pos..next, "");
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    if d.cursor_pos > 0 {
                        d.cursor_pos = d.input[..d.cursor_pos]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    if d.cursor_pos < d.input.len() {
                        d.cursor_pos = d.input[d.cursor_pos..]
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| d.cursor_pos + i)
                            .unwrap_or(d.input.len());
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    let before = &d.input[..d.cursor_pos];
                    d.cursor_pos = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                }
            }
            KeyEvent {
                code: KeyCode::End,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    let after = &d.input[d.cursor_pos..];
                    d.cursor_pos += after.find('\n').unwrap_or(after.len());
                }
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => {
                if let Some(ref mut d) = self.answer_dialog {
                    d.input.insert(d.cursor_pos, c);
                    d.cursor_pos += c.len_utf8();
                }
            }
            _ => {}
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

    /// Check if text at a given row contains bold cells matching the needle.
    pub fn has_bold_text_on_row(&mut self, row: u16, needle: &str) -> bool {
        self.terminal
            .draw(|f| ui::draw(f, &self.app))
            .unwrap();
        let buf = self.terminal.backend().buffer().clone();
        if row >= buf.area.height {
            return false;
        }
        // Build the row text and find the needle position
        let mut row_text = String::new();
        for x in 0..buf.area.width {
            row_text.push_str(buf[(x, row)].symbol());
        }
        if let Some(start) = row_text.find(needle) {
            // Check that at least the first non-space character in the needle range is bold
            for x in start..(start + needle.len()).min(buf.area.width as usize) {
                let cell = &buf[(x as u16, row)];
                if cell.symbol().trim().is_empty() {
                    continue;
                }
                return cell
                    .modifier
                    .contains(ratatui::style::Modifier::BOLD);
            }
        }
        false
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

    /// Wait for an integration cycle to complete (IntegrationComplete or Error).
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
                    let done = matches!(&msg, IntegratorMessage::IntegrationComplete)
                        || matches!(
                            &msg,
                            IntegratorMessage::StatusUpdate(s) if s.contains("Error")
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

    /// Wait for the status to contain a specific string, processing messages until it does.
    pub async fn wait_for_status_to_contain(&mut self, needle: &str) {
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                self.ui_rx.recv(),
            )
            .await
            {
                Ok(Some(msg)) => {
                    self.app.update_from_integrator(msg);
                    if self.app.status.contains(needle) {
                        return;
                    }
                }
                Ok(None) => return,
                Err(_) => continue,
            }
        }
        panic!(
            "Timed out waiting for status to contain '{}'. Current status: '{}'",
            needle, self.app.status
        );
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

    /// Advance the animation tick counter.
    pub fn tick(&mut self) {
        self.app.tick();
    }

    /// Scan row 0 for the most prominent non-default foreground color.
    pub fn status_line_color_name(&mut self) -> String {
        self.terminal
            .draw(|f| ui::draw(f, &self.app))
            .unwrap();
        let buf = self.terminal.backend().buffer().clone();
        for x in 0..buf.area.width {
            let cell = &buf[(x, 0)];
            if cell.symbol().trim().is_empty() {
                continue;
            }
            match cell.fg {
                Color::Yellow | Color::LightYellow => return "yellow".into(),
                Color::Red | Color::LightRed => return "red".into(),
                Color::Green | Color::LightGreen => return "green".into(),
                _ => continue,
            }
        }
        "none".into()
    }

    /// Get a snapshot of row 0 content for animation comparison.
    pub fn status_indicator_snapshot(&mut self) -> String {
        self.terminal
            .draw(|f| ui::draw(f, &self.app))
            .unwrap();
        let buf = self.terminal.backend().buffer().clone();
        let mut s = String::new();
        for x in 0..buf.area.width {
            let cell = &buf[(x, 0)];
            s.push_str(cell.symbol());
        }
        s.trim_end().to_string()
    }
}
