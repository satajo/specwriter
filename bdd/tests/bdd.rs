use cucumber::{given, then, when, World};
use specwriter::integrator::IntegratorConfig;
use specwriter::AppRunner;
use std::path::PathBuf;
use tempfile::TempDir;

const SCREEN_WIDTH: u16 = 100;
const SCREEN_HEIGHT: u16 = 30;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct SpecwriterWorld {
    runner: Option<AppRunner>,
    workdir: Option<TempDir>,
}

impl SpecwriterWorld {
    fn new() -> Self {
        Self {
            runner: None,
            workdir: None,
        }
    }

    fn workdir_path(&self) -> PathBuf {
        self.workdir.as_ref().unwrap().path().to_path_buf()
    }

    fn runner(&mut self) -> &mut AppRunner {
        self.runner.as_mut().expect("AppRunner not initialized")
    }

    fn start_with_config(&mut self, config: IntegratorConfig) {
        let runner = AppRunner::new(config, SCREEN_WIDTH, SCREEN_HEIGHT);
        self.runner = Some(runner);
    }
}

// --- GIVEN steps ---

#[given("a clean working directory")]
async fn clean_working_directory(world: &mut SpecwriterWorld) {
    world.workdir = Some(TempDir::new().unwrap());
}

#[given("the specwriter is running with a mock command")]
async fn running_with_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir.join("mock-claude.sh").to_string_lossy().into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
    };
    world.start_with_config(config);
}

#[given("the specwriter is running with a no-questions mock")]
async fn running_with_no_questions_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-no-questions.sh")
            .to_string_lossy()
            .into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
    };
    world.start_with_config(config);
}

#[given(expr = "the specwriter is running with command {string}")]
async fn running_with_command(world: &mut SpecwriterWorld, command: String) {
    let config = IntegratorConfig {
        command,
        args: Vec::new(),
        working_dir: world.workdir_path(),
    };
    world.start_with_config(config);
}

#[given("the specwriter is running with a failing mock command")]
async fn running_with_failing_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-fail.sh")
            .to_string_lossy()
            .into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
    };
    world.start_with_config(config);
}

#[given(expr = "SPEC.md already contains {string}")]
async fn spec_already_exists(world: &mut SpecwriterWorld, content: String) {
    let content = content.replace("\\n", "\n");
    std::fs::write(world.workdir_path().join("SPEC.md"), content).unwrap();
}

// --- WHEN steps ---

#[when(expr = "I type {string}")]
async fn type_text(world: &mut SpecwriterWorld, text: String) {
    world.runner().type_str(&text);
}

#[when("I press Ctrl+S")]
async fn press_ctrl_s(world: &mut SpecwriterWorld) {
    world.runner().submit();
}

#[when("I wait for integration to complete")]
async fn wait_for_integration(world: &mut SpecwriterWorld) {
    world.runner().wait_for_integration().await;
}

#[when("I wait for all integrations to finish")]
async fn wait_for_all_integrations(world: &mut SpecwriterWorld) {
    world.runner().wait_for_integration().await;
    world.runner().wait_until_idle().await;
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

#[then(expr = "the screen should show {string}")]
async fn screen_should_show(world: &mut SpecwriterWorld, expected: String) {
    let screen = world.runner().render();
    assert!(
        screen.contains(&expected),
        "Screen should contain '{}', but got:\n{}",
        expected,
        screen
    );
}

#[then(expr = "the screen should not show {string}")]
async fn screen_should_not_show(world: &mut SpecwriterWorld, expected: String) {
    let screen = world.runner().render();
    assert!(
        !screen.contains(&expected),
        "Screen should NOT contain '{}', but it does:\n{}",
        expected,
        screen
    );
}

#[then(expr = "the input area should show {string}")]
async fn input_area_should_show(world: &mut SpecwriterWorld, expected: String) {
    let screen = world.runner().render();
    // The input area is between the "Input" title and the help line
    let input_start = screen.find("Input").expect("Input area not found on screen");
    let input_section = &screen[input_start..];
    assert!(
        input_section.contains(&expected),
        "Input area should contain '{}', but got:\n{}",
        expected,
        input_section
    );
}

#[then(expr = "the input area should not show {string}")]
async fn input_area_should_not_show(world: &mut SpecwriterWorld, expected: String) {
    let screen = world.runner().render();
    let input_start = screen.find("Input").expect("Input area not found on screen");
    let help_start = screen[input_start..]
        .find("Ctrl+C")
        .map(|i| input_start + i)
        .unwrap_or(screen.len());
    let input_section = &screen[input_start..help_start];
    assert!(
        !input_section.contains(&expected),
        "Input area should NOT contain '{}', but it does:\n{}",
        expected,
        input_section
    );
}

#[tokio::main]
async fn main() {
    SpecwriterWorld::run("features").await;
}
