use cucumber::{given, then, when, World};
use specwriter::integrator::{IntegratorConfig, IntegratorMessage};
use specwriter::App;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::sync::mpsc;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct SpecwriterWorld {
    app: Option<App>,
    ui_rx: Option<mpsc::UnboundedReceiver<IntegratorMessage>>,
    workdir: Option<TempDir>,
    integration_count: usize,
    last_questions: Vec<String>,
    previous_questions: Option<Vec<String>>,
    mock_script: String,
}

impl SpecwriterWorld {
    fn new() -> Self {
        let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self {
            app: None,
            ui_rx: None,
            workdir: None,
            integration_count: 0,
            last_questions: Vec::new(),
            previous_questions: None,
            mock_script: bdd_dir.join("mock-claude.sh").to_string_lossy().into(),
        }
    }

    fn workdir_path(&self) -> PathBuf {
        self.workdir.as_ref().unwrap().path().to_path_buf()
    }

    fn init_app(&mut self) {
        let config = IntegratorConfig {
            command: self.mock_script.clone(),
            args: Vec::new(),
            working_dir: self.workdir_path(),
        };
        let (app, ui_rx) = App::with_config(config);
        self.app = Some(app);
        self.ui_rx = Some(ui_rx);
    }

    async fn drain_messages(&mut self) {
        let rx = self.ui_rx.as_mut().unwrap();
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
        let mut got_completion = false;

        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv()).await {
                Ok(Some(msg)) => {
                    let app = self.app.as_mut().unwrap();
                    match &msg {
                        IntegratorMessage::QuestionsUpdated(q) => {
                            self.previous_questions = Some(self.last_questions.clone());
                            self.last_questions = q.clone();
                        }
                        IntegratorMessage::StatusUpdate(s) => {
                            if s.contains("complete") || s.contains("Error") {
                                self.integration_count += 1;
                                got_completion = true;
                            }
                        }
                    }
                    app.update_from_integrator(msg);
                    if got_completion {
                        while let Ok(msg) = rx.try_recv() {
                            match &msg {
                                IntegratorMessage::QuestionsUpdated(q) => {
                                    self.previous_questions =
                                        Some(self.last_questions.clone());
                                    self.last_questions = q.clone();
                                }
                                _ => {}
                            }
                            app.update_from_integrator(msg);
                        }
                        break;
                    }
                }
                Ok(None) => break,
                Err(_) => continue,
            }
        }
    }
}

// --- GIVEN steps (all async for tokio runtime access) ---

#[given("a clean working directory")]
async fn clean_working_directory(world: &mut SpecwriterWorld) {
    world.workdir = Some(TempDir::new().unwrap());
}

#[given("the integrator is configured with a mock command")]
async fn configure_mock(world: &mut SpecwriterWorld) {
    world.init_app();
}

#[given(expr = "the integrator is configured with command {string}")]
async fn configure_custom_command(world: &mut SpecwriterWorld, command: String) {
    world.mock_script = command;
    world.init_app();
}

#[given("the integrator is configured with a failing mock command")]
async fn configure_failing_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    world.mock_script = bdd_dir
        .join("mock-claude-fail.sh")
        .to_string_lossy()
        .into();
    world.init_app();
}

#[given(expr = "SPEC.md already contains {string}")]
async fn spec_already_exists(world: &mut SpecwriterWorld, content: String) {
    let content = content.replace("\\n", "\n");
    std::fs::write(world.workdir_path().join("SPEC.md"), content).unwrap();
}

#[given("the mock command will not produce questions")]
async fn configure_no_questions_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    world.mock_script = bdd_dir
        .join("mock-claude-no-questions.sh")
        .to_string_lossy()
        .into();
    world.init_app();
}

// --- WHEN steps ---

#[when(expr = "I submit the message {string}")]
async fn submit_message(world: &mut SpecwriterWorld, message: String) {
    let app = world.app.as_mut().unwrap();
    app.input = message;
    app.cursor_pos = app.input.len();
    app.submit();
}

#[when(expr = "I immediately submit the message {string}")]
async fn immediately_submit_message(world: &mut SpecwriterWorld, message: String) {
    let app = world.app.as_mut().unwrap();
    app.input = message;
    app.cursor_pos = app.input.len();
    app.submit();
}

#[when("I wait for integration to complete")]
async fn wait_for_integration(world: &mut SpecwriterWorld) {
    world.drain_messages().await;
}

#[when(expr = "I type {string}")]
async fn type_text(world: &mut SpecwriterWorld, text: String) {
    let app = world.app.as_mut().unwrap();
    for c in text.chars() {
        app.insert_char(c);
    }
}

#[when("I submit")]
async fn submit_current(world: &mut SpecwriterWorld) {
    let app = world.app.as_mut().unwrap();
    app.submit();
}

// --- THEN steps ---

#[then("SPEC.md should exist")]
async fn spec_exists(world: &mut SpecwriterWorld) {
    let path = world.workdir_path().join("SPEC.md");
    assert!(path.exists(), "SPEC.md should exist at {:?}", path);
}

#[then(expr = "SPEC.md should contain {string}")]
async fn spec_contains(world: &mut SpecwriterWorld, expected: String) {
    let path = world.workdir_path().join("SPEC.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", path));
    assert!(
        content.contains(&expected),
        "SPEC.md should contain '{}', but got:\n{}",
        expected,
        content
    );
}

#[then(expr = "the status should be {string}")]
async fn status_is(world: &mut SpecwriterWorld, expected: String) {
    let app = world.app.as_ref().unwrap();
    assert_eq!(app.status, expected, "Status mismatch");
}

#[then(expr = "the status should contain {string}")]
async fn status_contains(world: &mut SpecwriterWorld, expected: String) {
    let app = world.app.as_ref().unwrap();
    assert!(
        app.status.contains(&expected),
        "Status '{}' should contain '{}'",
        app.status,
        expected
    );
}

#[then("no integration should have been triggered")]
async fn no_integration_triggered(world: &mut SpecwriterWorld) {
    assert_eq!(
        world.integration_count, 0,
        "No integration should have been triggered"
    );
}

#[then("the integrator should have received both messages in one batch")]
async fn messages_batched(world: &mut SpecwriterWorld) {
    assert_eq!(
        world.integration_count, 1,
        "Expected 1 integration (batched), got {}",
        world.integration_count
    );
}

#[then("I should see questions displayed")]
async fn questions_displayed(world: &mut SpecwriterWorld) {
    assert!(
        !world.last_questions.is_empty(),
        "Should have questions, but got none"
    );
}

#[then("there should be at most 3 questions")]
async fn at_most_3_questions(world: &mut SpecwriterWorld) {
    assert!(
        world.last_questions.len() <= 3,
        "Expected at most 3 questions, got {}",
        world.last_questions.len()
    );
}

#[then("the questions should have been updated")]
async fn questions_updated(world: &mut SpecwriterWorld) {
    assert!(
        world.previous_questions.is_some(),
        "Questions should have been updated at least once"
    );
}

#[then("I should see no questions displayed")]
async fn no_questions_displayed(world: &mut SpecwriterWorld) {
    let app = world.app.as_ref().unwrap();
    assert!(
        app.questions.is_empty(),
        "Expected no questions, got {:?}",
        app.questions
    );
}

#[tokio::main]
async fn main() {
    SpecwriterWorld::run("features").await;
}
