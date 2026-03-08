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
    pub model: Option<String>,
    pub web_search: bool,
    pub web_fetch: bool,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            command: "claude".into(),
            args: Vec::new(),
            working_dir,
            spec_filename: "SPEC.md".into(),
            model: None,
            web_search: false,
            web_fetch: false,
        }
    }
}

impl IntegratorConfig {
    pub fn from_settings(settings: &crate::settings::Settings, working_dir: PathBuf) -> Self {
        Self {
            command: settings.claude_command.clone(),
            args: Vec::new(),
            working_dir,
            spec_filename: settings.spec_filename.clone(),
            model: settings.model.clone(),
            web_search: settings.web_search,
            web_fetch: settings.web_fetch,
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
        let mut tools = format!("Read,Edit({}),Write({})", self.spec_filename, self.spec_filename);
        if self.web_search {
            tools.push_str(",WebSearch");
        }
        if self.web_fetch {
            tools.push_str(",WebFetch");
        }
        let mut args = vec![
            "--print".into(),
            "--allowedTools".into(),
            tools,
        ];
        if let Some(ref model) = self.model {
            args.push("--model".into());
            args.push(model.clone());
        }
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

/// Format messages as a numbered list, regardless of batch size.
fn format_messages(messages: &[String]) -> String {
    messages
        .iter()
        .enumerate()
        .map(|(i, msg)| format!("Message {}:\n{}", i + 1, msg))
        .collect::<Vec<_>>()
        .join("\n\n")
}

async fn integrator_loop(
    mut rx: mpsc::UnboundedReceiver<String>,
    ui_tx: mpsc::UnboundedSender<IntegratorMessage>,
    config: IntegratorConfig,
) {
    let spec_file = config.spec_path();
    let mut session_id = Uuid::new_v4().to_string();
    let mut first_call = true;
    // Buffer for messages that arrived during command execution
    let mut pending: Vec<String> = Vec::new();

    loop {
        // Wait for at least one message (from pending buffer or channel)
        if pending.is_empty() {
            match rx.recv().await {
                Some(m) => pending.push(m),
                None => return,
            }
        }

        // Drain any immediately available messages
        while let Ok(msg) = rx.try_recv() {
            pending.push(msg);
        }

        // Take the entire batch
        let batch: Vec<String> = pending.drain(..).collect();
        let _ = ui_tx.send(IntegratorMessage::StatusUpdate("Integrating".into()));

        let message = format_messages(&batch);
        let spec_is_empty = !spec_file.exists()
            || std::fs::read_to_string(&spec_file)
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);

        let sf = &config.spec_filename;
        let prompt = if !spec_is_empty {
            IntegratePrompt { sf, message: &message }.to_string()
        } else {
            CreateSpecPrompt { sf, message: &message }.to_string()
        };

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
                        pending.push(msg);
                        let status = format!("Integrating ({} in queue)", pending.len());
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

                // Drain any last-moment arrivals
                while let Ok(msg) = rx.try_recv() {
                    pending.push(msg);
                }

                if pending.is_empty() {
                    let _ = ui_tx.send(IntegratorMessage::IntegrationComplete);
                }
            }
            Err(e) => {
                if !first_call {
                    // Retry with a fresh session, including any new messages
                    session_id = Uuid::new_v4().to_string();
                    first_call = true;

                    while let Ok(msg) = rx.try_recv() {
                        pending.push(msg);
                    }

                    // Rebuild batch: original messages + new arrivals
                    let mut retry_messages = batch;
                    retry_messages.append(&mut pending);

                    let retry_message = format_messages(&retry_messages);
                    let retry_prompt = if !spec_is_empty {
                        IntegratePrompt { sf, message: &retry_message }.to_string()
                    } else {
                        CreateSpecPrompt { sf, message: &retry_message }.to_string()
                    };

                    let fresh_args = vec![
                        "--session-id".to_string(),
                        session_id.clone(),
                    ];
                    let retry = run_command(&config, &fresh_args, &retry_prompt).await;
                    match retry {
                        Ok(_) => {
                            first_call = false;
                            let questions = scan_questions(&spec_file);
                            let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(questions));

                            while let Ok(msg) = rx.try_recv() {
                                pending.push(msg);
                            }

                            if pending.is_empty() {
                                let _ = ui_tx.send(IntegratorMessage::IntegrationComplete);
                            }
                        }
                        Err(e2) => {
                            // Double failure: drain and discard everything
                            while let Ok(_) = rx.try_recv() {}
                            pending.clear();
                            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
                                format!("Error! {e2}"),
                            ));
                        }
                    }
                } else {
                    // First call failed — no recovery possible
                    while let Ok(_) = rx.try_recv() {}
                    pending.clear();
                    let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!("Error! {e}")));
                }
            }
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
    // Clamp priority to 1-5 range for display; values > 5 from legacy specs map to 5
    let priority = priority.min(5).max(1);
    Some(Question { id, text, body: String::new(), priority, solutions: Vec::new() })
}
