use cucumber::{given, then, when, World};
use specwriter::integrator::IntegratorConfig;
use specwriter::AppRunner;
use std::path::{Path, PathBuf};
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

fn search_spec_files(spec_dir: &Path, needle: &str) -> bool {
    if !spec_dir.exists() {
        return false;
    }
    search_dir(spec_dir, needle)
}

fn search_dir(dir: &Path, needle: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if search_dir(&path, needle) {
                        return true;
                    }
                } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains(needle) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
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

#[given("the specwriter is running with a slow mock command")]
async fn running_with_slow_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-slow.sh")
            .to_string_lossy()
            .into(),
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


#[given("the specwriter is running with a nine-questions mock")]
async fn running_with_nine_questions_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-nine-questions.sh")
            .to_string_lossy()
            .into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
    };
    world.start_with_config(config);
}

#[given("the specwriter is running with a prioritized mock")]
async fn running_with_prioritized_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-prioritized.sh")
            .to_string_lossy()
            .into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
    };
    world.start_with_config(config);
}

#[given(expr = "the spec README already contains {string}")]
async fn spec_readme_already_contains(world: &mut SpecwriterWorld, content: String) {
    let content = content.replace("\\n", "\n");
    let spec_dir = world.workdir_path().join("spec");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("README.md"), content).unwrap();
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

#[when(expr = "I wait for status to contain {string}")]
async fn wait_for_status(world: &mut SpecwriterWorld, needle: String) {
    world.runner().wait_for_status_to_contain(&needle).await;
}

// --- THEN steps ---

#[then("the spec README should exist")]
async fn spec_readme_exists(world: &mut SpecwriterWorld) {
    let path = world.workdir_path().join("spec").join("README.md");
    assert!(path.exists(), "spec/README.md should exist at {:?}", path);
}

#[then("the spec directory should not exist")]
async fn spec_dir_not_exists(world: &mut SpecwriterWorld) {
    let path = world.workdir_path().join("spec");
    assert!(!path.exists(), "spec/ should NOT exist at {:?}", path);
}

#[then(expr = "the spec should contain {string}")]
async fn spec_contains(world: &mut SpecwriterWorld, expected: String) {
    let spec_dir = world.workdir_path().join("spec");
    assert!(
        search_spec_files(&spec_dir, &expected),
        "No spec file contains '{}'",
        expected
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

#[then(expr = "question {string} should appear before {string} on screen")]
async fn question_appears_before(world: &mut SpecwriterWorld, first: String, second: String) {
    let screen = world.runner().render();
    let first_q = format!("{}.", first);
    let second_q = format!("{}.", second);
    let pos_first = screen.find(&first_q).unwrap_or_else(|| {
        panic!("'{}' not found on screen:\n{}", first_q, screen)
    });
    let pos_second = screen.find(&second_q).unwrap_or_else(|| {
        panic!("'{}' not found on screen:\n{}", second_q, screen)
    });
    assert!(
        pos_first < pos_second,
        "Expected '{}' before '{}' on screen, but positions are {} vs {}:\n{}",
        first_q, second_q, pos_first, pos_second, screen
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

#[then(expr = "the status indicator should be {word}")]
async fn status_indicator_color(world: &mut SpecwriterWorld, expected_color: String) {
    let actual = world.runner().status_indicator_color_name();
    assert!(
        actual == expected_color,
        "Expected status indicator to be '{}', but got '{}'",
        expected_color,
        actual
    );
}

#[when("time passes")]
async fn time_passes(world: &mut SpecwriterWorld) {
    // Advance several animation frames
    for _ in 0..15 {
        world.runner().tick();
    }
}

#[then("the status indicator should have animated")]
async fn indicator_should_have_animated(world: &mut SpecwriterWorld) {
    let runner = world.runner();
    let before = runner.status_indicator_snapshot();
    for _ in 0..5 {
        runner.tick();
    }
    let after = runner.status_indicator_snapshot();
    assert!(
        before != after,
        "Status indicator should have animated, but stayed the same: {}",
        before
    );
}

#[then(expr = "{string} should succeed")]
async fn command_should_succeed(_world: &mut SpecwriterWorld, command: String) {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let parts: Vec<&str> = command.split_whitespace().collect();
    let output = tokio::process::Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(&workspace_root)
        .output()
        .await
        .unwrap_or_else(|e| panic!("Failed to run '{}': {}", command, e));
    assert!(
        output.status.success(),
        "'{}' failed:\n{}",
        command,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[then(expr = "the nix build output should contain a {string} binary")]
async fn nix_output_contains_binary(_world: &mut SpecwriterWorld, name: String) {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let binary = workspace_root.join("result").join("bin").join(&name);
    assert!(
        binary.exists(),
        "Expected binary at {:?} but it doesn't exist",
        binary
    );
}

#[tokio::main]
async fn main() {
    SpecwriterWorld::run("features").await;
}
