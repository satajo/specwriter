use askama::Template;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "integrate.md")]
struct IntegratePrompt<'a> {
    sf: &'a str,
    message: &'a str,
}

#[derive(Template)]
#[template(path = "create_spec.md")]
struct CreateSpecPrompt<'a> {
    sf: &'a str,
    message: &'a str,
}
#[derive(Debug, Clone)]
pub enum IntegratorMessage {
    QuestionsUpdated(Vec<Question>),
    StatusUpdate(String),
    IntegrationComplete,
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Question {
    pub id: usize,
    pub text: String,
    pub body: String,
    pub priority: u8,
    pub solutions: Vec<Solution>,
}

#[derive(Debug, Clone)]
pub struct IntegratorConfig {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub spec_filename: String,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            command: "claude".into(),
            args: Vec::new(),
            working_dir,
            spec_filename: "SPEC.md".into(),
        }
    }
}

impl IntegratorConfig {
    pub fn spec_path(&self) -> PathBuf {
        self.working_dir.join(&self.spec_filename)
    }
}

impl IntegratorConfig {
    /// Build CLI args with properly scoped tool permissions.
    /// Read is scoped to the working directory, Edit/Write only to the spec file.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            "--print".into(),
            "--allowedTools".into(),
            format!("Read,Edit({}),Write({})", self.spec_filename, self.spec_filename),
        ];
        args.extend(self.args.iter().cloned());
        args
    }
}

#[derive(Debug)]
pub struct IntegratorHandle {
    tx: mpsc::UnboundedSender<String>,
    working_dir: PathBuf,
    spec_filename: String,
}

impl IntegratorHandle {
    pub fn new(
        ui_tx: mpsc::UnboundedSender<IntegratorMessage>,
        config: IntegratorConfig,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let working_dir = config.working_dir.clone();
        let spec_filename = config.spec_filename.clone();
        tokio::spawn(integrator_loop(rx, ui_tx, config));
        Self { tx, working_dir, spec_filename }
    }

    pub fn send(&self, message: String) {
        let _ = self.tx.send(message);
    }

    pub fn spec_path(&self) -> PathBuf {
        self.working_dir.join(&self.spec_filename)
    }

    pub fn spec_filename(&self) -> &str {
        &self.spec_filename
    }
}

async fn integrator_loop(
    mut rx: mpsc::UnboundedReceiver<String>,
    ui_tx: mpsc::UnboundedSender<IntegratorMessage>,
    config: IntegratorConfig,
) {
    let spec_file = config.spec_path();
    // Session ID for CLI session reuse across integrations
    let mut session_id = Uuid::new_v4().to_string();

    // Main integration loop
    let mut first_call = true;
    loop {
        let msg = match rx.recv().await {
            Some(m) => m,
            None => return,
        };
        let mut queue = vec![msg];

        // Brief window to collect rapid submissions before processing starts
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        while let Ok(msg) = rx.try_recv() {
            queue.push(msg);
        }

        // Process each message one at a time
        let mut i = 0;
        let mut errored = false;
        while i < queue.len() {
            // Drain any messages that arrived since last iteration
            while let Ok(msg) = rx.try_recv() {
                queue.push(msg);
            }

            let waiting = queue.len() - i - 1;
            let status = if waiting > 0 {
                format!("Integrating ({} in queue)", waiting)
            } else {
                "Integrating".into()
            };
            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(status));

            let spec_is_empty = !spec_file.exists()
                || std::fs::read_to_string(&spec_file)
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true);
            let message = &queue[i];

            let sf = &config.spec_filename;
            let prompt = if !spec_is_empty {
                IntegratePrompt { sf, message }.to_string()
            } else {
                CreateSpecPrompt { sf, message }.to_string()
            };

            // Build session args for CLI session reuse
            let extra_args: Vec<String> = if first_call {
                vec!["--session-id".to_string(), session_id.clone()]
            } else {
                vec!["--resume".to_string(), session_id.clone()]
            };

            // Run command while monitoring for new submissions
            let command_future = run_command(&config, &extra_args, &prompt);
            tokio::pin!(command_future);

            let result = loop {
                tokio::select! {
                    result = &mut command_future => break result,
                    msg = rx.recv() => {
                        if let Some(msg) = msg {
                            queue.push(msg);
                            let waiting = queue.len() - i - 1;
                            let status = format!("Integrating ({} in queue)", waiting);
                            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(status));
                        }
                    }
                }
            };

            match result {
                Ok(_) => {
                    first_call = false;
                    let questions = scan_questions(&spec_file);
                    let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(questions));
                }
                Err(e) => {
                    // If this was a --resume call, try recovering with a fresh session
                    if !first_call {
                        session_id = Uuid::new_v4().to_string();
                        first_call = true;
                        let fresh_args = vec![
                            "--session-id".to_string(),
                            session_id.clone(),
                        ];
                        let retry = run_command(&config, &fresh_args, &prompt).await;
                        match retry {
                            Ok(_) => {
                                first_call = false;
                                let questions = scan_questions(&spec_file);
                                let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(questions));
                            }
                            Err(e2) => {
                                while let Ok(_) = rx.try_recv() {}
                                let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
                                    format!("Error! {e2}"),
                                ));
                                errored = true;
                                break;
                            }
                        }
                    } else {
                        // First call failed — no recovery possible
                        while let Ok(_) = rx.try_recv() {}
                        let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!("Error! {e}")));
                        errored = true;
                        break;
                    }
                }
            }

            i += 1;
        }

        if !errored {
            let _ = ui_tx.send(IntegratorMessage::IntegrationComplete);
        }
    }
}


async fn run_command(config: &IntegratorConfig, extra_args: &[String], prompt: &str) -> Result<String, String> {
    let output = Command::new(&config.command)
        .args(&config.build_args())
        .args(extra_args)
        .arg("-p")
        .arg(prompt)
        .current_dir(&config.working_dir)
        .env("CLAUDE_CODE_SIMPLE", "1")
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Failed to run {}: {e}", config.command))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let first_line = stderr.lines().next().unwrap_or("").trim();
        let description = if !first_line.is_empty() {
            first_line.to_string()
        } else if let Some(code) = output.status.code() {
            format!("Exit code {} with no message", code)
        } else {
            "Unknown reason".to_string()
        };
        return Err(description);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Scan SPEC.md for questions under ## Questions headings.
/// Returns Questions sorted by priority (highest first).
pub fn scan_questions(spec_file: &Path) -> Vec<Question> {
    let mut questions = Vec::new();
    if let Ok(content) = std::fs::read_to_string(spec_file) {
        parse_questions_from_content(&content, &mut questions);
    }
    questions.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.id.cmp(&b.id)));
    questions
}

fn parse_questions_from_content(content: &str, questions: &mut Vec<Question>) {
    let mut in_questions_section = false;
    let mut current_question: Option<Question> = None;
    let mut body_lines: Vec<String> = Vec::new();
    let mut current_solution: Option<Solution> = None;
    let mut solution_body_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Questions" {
            in_questions_section = true;
            continue;
        }
        if in_questions_section && trimmed.starts_with("## ") && trimmed != "## Questions" {
            break;
        }
        if !in_questions_section {
            continue;
        }
        if let Some(q) = parse_question_heading(trimmed) {
            // Flush current solution into previous question
            if let Some(mut sol) = current_solution.take() {
                sol.body = solution_body_lines.join("\n").trim().to_string();
                if let Some(ref mut prev) = current_question {
                    prev.solutions.push(sol);
                }
                solution_body_lines.clear();
            }
            // Flush previous question
            if let Some(mut prev) = current_question.take() {
                if prev.solutions.is_empty() {
                    prev.body = body_lines.join("\n").trim().to_string();
                }
                questions.push(prev);
            }
            body_lines.clear();
            current_question = Some(q);
        } else if trimmed.starts_with("#### ") && current_question.is_some() {
            // Flush previous solution
            if let Some(mut sol) = current_solution.take() {
                sol.body = solution_body_lines.join("\n").trim().to_string();
                if let Some(ref mut q) = current_question {
                    q.solutions.push(sol);
                }
                solution_body_lines.clear();
            } else {
                // First solution — finalize question body from lines collected so far
                if let Some(ref mut q) = current_question {
                    q.body = body_lines.join("\n").trim().to_string();
                }
            }
            let title = trimmed.strip_prefix("#### ").unwrap().trim().to_string();
            current_solution = Some(Solution { title, body: String::new() });
        } else if current_solution.is_some() {
            solution_body_lines.push(line.to_string());
        } else if current_question.is_some() {
            body_lines.push(line.to_string());
        }
    }
    // Flush last solution
    if let Some(mut sol) = current_solution.take() {
        sol.body = solution_body_lines.join("\n").trim().to_string();
        if let Some(ref mut q) = current_question {
            q.solutions.push(sol);
        }
    }
    // Flush last question
    if let Some(mut q) = current_question.take() {
        if q.solutions.is_empty() {
            q.body = body_lines.join("\n").trim().to_string();
        }
        questions.push(q);
    }
}

/// Parse a `### Q<id> (p<priority>): <title>` heading line.
fn parse_question_heading(line: &str) -> Option<Question> {
    let rest = line.strip_prefix("### Q")?;
    let colon_pos = rest.find(':')?;
    let before_colon = rest[..colon_pos].trim();

    // Parse "ID (pPRIORITY)" or just "ID"
    let (id, priority) = if let Some(paren_start) = before_colon.find('(') {
        let id: usize = before_colon[..paren_start].trim().parse().ok()?;
        let inside = before_colon[paren_start + 1..].trim_end_matches(')').trim();
        let priority: u8 = inside.strip_prefix('p')?.parse().ok()?;
        (id, priority)
    } else {
        let id: usize = before_colon.parse().ok()?;
        (id, 5) // default priority
    };

    let text = rest[colon_pos + 1..].trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(Question { id, text, body: String::new(), priority, solutions: Vec::new() })
}
