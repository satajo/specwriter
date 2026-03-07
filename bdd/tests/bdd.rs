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
        spec_dir_name: "specs".into(),
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
        spec_dir_name: "specs".into(),
    };
    world.start_with_config(config);
}

#[given(expr = "the specwriter is running with command {string}")]
async fn running_with_command(world: &mut SpecwriterWorld, command: String) {
    let config = IntegratorConfig {
        command,
        args: Vec::new(),
        working_dir: world.workdir_path(),
        spec_dir_name: "specs".into(),
    };
    world.start_with_config(config);
}

#[given(expr = "the specwriter is running with specs dir {string} and a mock command")]
async fn running_with_custom_specs_dir(world: &mut SpecwriterWorld, spec_dir: String) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir.join("mock-claude.sh").to_string_lossy().into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
        spec_dir_name: spec_dir,
    };
    world.start_with_config(config);
}

#[given("the specwriter is running with a session-expiry mock")]
async fn running_with_session_expiry_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-session-expiry.sh")
            .to_string_lossy()
            .into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
        spec_dir_name: "specs".into(),
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
        spec_dir_name: "specs".into(),
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
        spec_dir_name: "specs".into(),
    };
    world.start_with_config(config);
}


#[given("the specwriter is running with a silent-fail mock command")]
async fn running_with_silent_fail_mock(world: &mut SpecwriterWorld) {
    let bdd_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = IntegratorConfig {
        command: bdd_dir
            .join("mock-claude-silent-fail.sh")
            .to_string_lossy()
            .into(),
        args: Vec::new(),
        working_dir: world.workdir_path(),
        spec_dir_name: "specs".into(),
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
        spec_dir_name: "specs".into(),
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
        spec_dir_name: "specs".into(),
    };
    world.start_with_config(config);
}

#[given(expr = "the spec README already contains {string}")]
async fn spec_readme_already_contains(world: &mut SpecwriterWorld, content: String) {
    let content = content.replace("\\n", "\n");
    let spec_dir = world.workdir_path().join("specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("README.md"), content).unwrap();
}

#[given("the spec README already contains 20 questions")]
async fn spec_readme_contains_20_questions(world: &mut SpecwriterWorld) {
    let spec_dir = world.workdir_path().join("specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    let mut content = String::from("# App\n\n## Questions\n\n");
    for i in 1..=20 {
        content.push_str(&format!(
            "### Q{} (p5): Question number {}?\n\nBody for question {}.\n\n",
            i, i, i
        ));
    }
    std::fs::write(spec_dir.join("README.md"), content).unwrap();
}

#[given(expr = "specs are in directory {string} with content {string}")]
async fn specs_in_custom_dir(world: &mut SpecwriterWorld, dir: String, content: String) {
    let content = content.replace("\\n", "\n");
    let spec_dir = world.workdir_path().join(&dir);
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

#[when("I press Tab")]
async fn press_tab(world: &mut SpecwriterWorld) {
    world
        .runner()
        .send_key(specwriter::KeyCode::Tab, specwriter::KeyModifiers::NONE);
}

#[when("I press Enter")]
async fn press_enter(world: &mut SpecwriterWorld) {
    world
        .runner()
        .send_key(specwriter::KeyCode::Enter, specwriter::KeyModifiers::NONE);
}

#[when("I press Down")]
async fn press_down(world: &mut SpecwriterWorld) {
    world
        .runner()
        .send_key(specwriter::KeyCode::Down, specwriter::KeyModifiers::NONE);
}

#[when("I press Up")]
async fn press_up(world: &mut SpecwriterWorld) {
    world
        .runner()
        .send_key(specwriter::KeyCode::Up, specwriter::KeyModifiers::NONE);
}

#[when(expr = "I press Down {int} times")]
async fn press_down_n_times(world: &mut SpecwriterWorld, n: usize) {
    for _ in 0..n {
        world
            .runner()
            .send_key(specwriter::KeyCode::Down, specwriter::KeyModifiers::NONE);
    }
}

#[when(expr = "I press Up {int} times")]
async fn press_up_n_times(world: &mut SpecwriterWorld, n: usize) {
    for _ in 0..n {
        world
            .runner()
            .send_key(specwriter::KeyCode::Up, specwriter::KeyModifiers::NONE);
    }
}

#[when("I press Ctrl+C")]
async fn press_ctrl_c(world: &mut SpecwriterWorld) {
    world
        .runner()
        .send_key(specwriter::KeyCode::Char('c'), specwriter::KeyModifiers::CONTROL);
}

#[when("I press Esc")]
async fn press_esc(world: &mut SpecwriterWorld) {
    world
        .runner()
        .send_key(specwriter::KeyCode::Esc, specwriter::KeyModifiers::NONE);
}

#[given("I switch to the questions tab")]
#[when("I switch to the questions tab")]
async fn switch_to_questions_tab(world: &mut SpecwriterWorld) {
    use specwriter::ActiveTab;
    // Press Tab until we're on the Questions tab
    for _ in 0..2 {
        if world.runner().app.active_tab == ActiveTab::Questions {
            return;
        }
        world
            .runner()
            .send_key(specwriter::KeyCode::Tab, specwriter::KeyModifiers::NONE);
    }
}

#[when("I switch to the text input tab")]
async fn switch_to_text_input_tab(world: &mut SpecwriterWorld) {
    use specwriter::ActiveTab;
    for _ in 0..2 {
        if world.runner().app.active_tab == ActiveTab::TextInput {
            return;
        }
        world
            .runner()
            .send_key(specwriter::KeyCode::Tab, specwriter::KeyModifiers::NONE);
    }
}

// --- THEN steps ---

#[then("the spec README should exist")]
async fn spec_readme_exists(world: &mut SpecwriterWorld) {
    let path = world.workdir_path().join("specs").join("README.md");
    assert!(path.exists(), "spec/README.md should exist at {:?}", path);
}

#[then("the spec directory should not exist")]
async fn spec_dir_not_exists(world: &mut SpecwriterWorld) {
    let path = world.workdir_path().join("specs");
    assert!(!path.exists(), "spec/ should NOT exist at {:?}", path);
}

#[then(expr = "the spec should contain {string}")]
async fn spec_contains(world: &mut SpecwriterWorld, expected: String) {
    let spec_dir = world.workdir_path().join("specs");
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

#[then(expr = "the detail panel should show {string}")]
async fn detail_panel_should_show(world: &mut SpecwriterWorld, expected: String) {
    let screen = world.runner().render();
    let detail_start = screen.find("Details").unwrap_or_else(|| {
        panic!("Details panel not found on screen:\n{}", screen)
    });
    let detail_section = &screen[detail_start..];
    assert!(
        detail_section.contains(&expected),
        "Detail panel should contain '{}', but got:\n{}",
        expected,
        detail_section
    );
}

#[then(expr = "the question list should show {string}")]
async fn question_list_should_show(world: &mut SpecwriterWorld, expected: String) {
    let screen = world.runner().render();
    // The question list is between the tab bar and the "Details" panel
    let list_end = screen.find("Details").unwrap_or(screen.len());
    let list_section = &screen[..list_end];
    assert!(
        list_section.contains(&expected),
        "Question list should contain '{}', but got:\n{}",
        expected,
        list_section
    );
}

#[then("the app should have quit")]
async fn app_should_have_quit(world: &mut SpecwriterWorld) {
    assert!(
        world.runner().app.should_quit,
        "Expected the app to have quit, but it hasn't"
    );
}

#[then("the app should not have quit")]
async fn app_should_not_have_quit(world: &mut SpecwriterWorld) {
    assert!(
        !world.runner().app.should_quit,
        "Expected the app to NOT have quit, but it has"
    );
}

#[then(expr = "question {string} should appear before {string} on screen")]
async fn question_appears_before(world: &mut SpecwriterWorld, first: String, second: String) {
    let screen = world.runner().render();
    let first_q = format!("{} (p", first);
    let second_q = format!("{} (p", second);
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

#[then(expr = "the status line should contain {word} text")]
async fn status_line_should_contain_colored_text(
    world: &mut SpecwriterWorld,
    expected_color: String,
) {
    let actual = world.runner().status_line_color_name();
    assert!(
        actual == expected_color,
        "Expected status line to contain '{}' text, but found '{}'",
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

fn get_row(world: &mut SpecwriterWorld, row: usize) -> String {
    let screen = world.runner().render();
    let lines: Vec<&str> = screen.split('\n').collect();
    assert!(
        row >= 1 && row <= lines.len(),
        "Row {} out of range (screen has {} rows)",
        row,
        lines.len()
    );
    lines[row - 1].to_string()
}

#[then(expr = "row {int} should start with the status icon followed by {string}")]
async fn row_starts_with_status_icon(world: &mut SpecwriterWorld, row: usize, text: String) {
    let line = get_row(world, row);
    // Status icon is a multi-byte unicode char followed by a space
    let after_icon = line.trim_start().chars().next().map(|c| !c.is_ascii()).unwrap_or(false);
    assert!(
        after_icon,
        "Row {} should start with a status icon, but got: '{}'",
        row, line
    );
    assert!(
        line.contains(&text),
        "Row {} should contain '{}', but got: '{}'",
        row, text, line
    );
}

#[then(expr = "row {int} should be blank")]
async fn row_should_be_blank(world: &mut SpecwriterWorld, row: usize) {
    let line = get_row(world, row);
    assert!(
        line.trim().is_empty(),
        "Row {} should be blank, but got: '{}'",
        row, line
    );
}

#[then(expr = "row {int} should start with {string}")]
async fn row_starts_with(world: &mut SpecwriterWorld, row: usize, prefix: String) {
    let line = get_row(world, row);
    assert!(
        line.starts_with(&prefix),
        "Row {} should start with '{}', but got: '{}'",
        row, prefix, line
    );
}

fn is_box_drawing(c: char) -> bool {
    ('\u{2500}'..='\u{257F}').contains(&c)
}

#[then(expr = "row {int} should contain no box-drawing characters")]
async fn row_has_no_box_drawing(world: &mut SpecwriterWorld, row: usize) {
    let line = get_row(world, row);
    let offending: Vec<char> = line.chars().filter(|c| is_box_drawing(*c)).collect();
    assert!(
        offending.is_empty(),
        "Row {} should contain no box-drawing characters, but found {:?}: '{}'",
        row, offending, line
    );
}

#[then(expr = "row {int} should start with a corner box-drawing character followed by horizontal lines")]
async fn row_starts_with_corner_then_horizontal(world: &mut SpecwriterWorld, row: usize) {
    let line = get_row(world, row);
    let mut chars = line.chars();
    let first = chars.next().unwrap_or(' ');
    // Corner characters: ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ╭ ╮ ╯ ╰ etc.
    let corners = [
        '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '╭', '╮', '╯', '╰',
    ];
    assert!(
        corners.contains(&first),
        "Row {} should start with a corner character, but starts with '{}' (U+{:04X}): '{}'",
        row, first, first as u32, line
    );
    let second = chars.next().unwrap_or(' ');
    // Horizontal line characters: ─ ━ ═
    let horizontals = ['─', '━', '═'];
    assert!(
        horizontals.contains(&second),
        "Row {} second character should be a horizontal line, but got '{}' (U+{:04X}): '{}'",
        row, second, second as u32, line
    );
}

#[then(expr = "row {int} should start with a vertical border then {string}")]
async fn row_starts_with_border_then_text(world: &mut SpecwriterWorld, row: usize, text: String) {
    let line = get_row(world, row);
    let first = line.chars().next().unwrap_or(' ');
    let verticals = ['│', '┃', '║'];
    assert!(
        verticals.contains(&first),
        "Row {} should start with a vertical border, but starts with '{}' (U+{:04X}): '{}'",
        row, first, first as u32, line
    );
    // After the border and a space, the text should follow
    let after_border = line[first.len_utf8()..].trim_start();
    assert!(
        after_border.starts_with(&text),
        "Row {} after border should start with '{}', but got: '{}'",
        row, text, after_border
    );
}

#[then("the active tab title should be bold")]
async fn active_tab_should_be_bold(world: &mut SpecwriterWorld) {
    let tab_name = match world.runner().app.active_tab {
        specwriter::ActiveTab::TextInput => "Text Input",
        specwriter::ActiveTab::Questions => "Open Questions",
    };
    // Tab labels are on row index 2 (row 3, zero-indexed)
    assert!(
        world.runner().has_bold_text_on_row(2, tab_name),
        "Active tab '{}' should be bold on row 3",
        tab_name
    );
}

#[tokio::main]
async fn main() {
    SpecwriterWorld::run("features").await;
}
