use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum IntegratorMessage {
    QuestionsUpdated(Vec<(usize, String)>),
    StatusUpdate(String),
    IntegrationComplete,
}

#[derive(Debug, Clone)]
pub struct IntegratorConfig {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        Self {
            command: "claude".into(),
            args: vec![
                "--print".into(),
                "--allowedTools".into(),
                "Edit,Read,Write".into(),
                "-p".into(),
            ],
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

#[derive(Debug)]
pub struct IntegratorHandle {
    tx: mpsc::UnboundedSender<String>,
}

impl IntegratorHandle {
    pub fn new(
        ui_tx: mpsc::UnboundedSender<IntegratorMessage>,
        config: IntegratorConfig,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(integrator_loop(rx, ui_tx, config));
        Self { tx }
    }

    pub fn send(&self, message: String) {
        let _ = self.tx.send(message);
    }
}

async fn integrator_loop(
    mut rx: mpsc::UnboundedReceiver<String>,
    ui_tx: mpsc::UnboundedSender<IntegratorMessage>,
    config: IntegratorConfig,
) {
    let spec_dir = config.working_dir.join("spec");

    // Seed questions from existing spec
    let readme_path = spec_dir.join("README.md");
    if readme_path.exists() {
        let content = std::fs::read_to_string(&readme_path).unwrap_or_default();
        if !content.trim().is_empty() {
            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
                "Loading existing specs...".into(),
            ));

            let prompt = r#"You are a requirements integrator. Review the existing spec knowledge base under the spec/ directory and generate initial clarifying questions based on its current content.

Read spec/README.md to orient yourself, then read whatever other spec files you deem relevant.

INLINE QUESTIONS:
Embed your questions directly in the relevant spec files using this format (one per line):
?Q{id}: {question text}

Place each question near the content it relates to. Assign sequential IDs starting from 1. Generate up to 9 questions, focusing on the most important things to clarify.

Do NOT output questions to stdout — embed them in the spec files only."#;

            if let Err(e) = run_command(&config, prompt).await {
                let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!("Error: {e}")));
            }

            let questions = scan_inline_questions(&spec_dir);
            let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(questions));
            let _ = ui_tx.send(IntegratorMessage::IntegrationComplete);
        }
    }

    // Main integration loop
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
                format!("Integrating ({} in queue)...", waiting)
            } else {
                "Integrating...".into()
            };
            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(status));

            // Create spec directory if it doesn't exist
            if !spec_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&spec_dir) {
                    let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!(
                        "Error: Failed to create spec directory: {e}"
                    )));
                    errored = true;
                    break;
                }
            }

            let spec_is_empty = !readme_path.exists()
                || std::fs::read_to_string(&readme_path)
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true);
            let message = &queue[i];

            let prompt = if !spec_is_empty {
                format!(
                    r#"You are a requirements integrator managing a spec knowledge base under the spec/ directory.

Read spec/README.md to orient yourself, then read whatever other spec files you deem relevant for integrating the following user message.

RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive knowledge base, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Exercise judgment about the weight and nature of each input. Not all inputs are equal — some are core requirements, others are asides or loosely structured thoughts. Summarize, condense, or reframe as appropriate to maintain coherence and quality, while always preserving intent.
- If the input seems unrelated to existing content, create a new topic area or file as appropriate.
- Integrate autonomously — do not ask the user to approve the output. If something is wrong, the user will submit corrective input.

SPEC STRUCTURE:
- All spec files live under spec/
- spec/README.md is the primary entrypoint — it should always exist and be useful
- For a small spec, README.md may contain all the substance; as the knowledge base grows, it shifts toward an index role pointing readers to the right spec files
- Prefer keeping README.md self-contained and substantive — err on the side of a longer README over premature splitting
- You may create additional spec files or subdirectories as needed
- Use prose and lists only — no diagrams, tables, or non-textual content
- Stick to basic Markdown — headings, paragraphs, lists, bold/italic, links

SPEC ORGANIZATION:
- You own the organization of spec/ — create, split, merge, rename files as you see fit
- Use whatever structure and naming makes sense for the material
- Clean up empty or low-value files
- READMEs should add value through descriptions and context, not mirror directory structure

CODEBASE CONTEXT:
You have read access to the project where this tool is running. Gather whatever codebase context you need to make sense of the user's requirements — look at relevant files, understand the domain, terminology, and existing structure. Do this autonomously without requiring user guidance.

INLINE QUESTIONS:
Embed clarifying questions directly in spec files using this format (one per line):
?Q{{id}}: {{question text}}

Place each question near the content it relates to. Questions are global across the knowledge base.
- Keep questions that are still relevant and unanswered (preserve their IDs)
- Remove questions that have been answered or are no longer relevant
- Add new questions with IDs higher than any existing question ID
- Maximum 9 questions across all spec files
- Each question should be self-contained — understandable without cross-referencing
- If input contradicts existing spec content, integrate it and optionally raise a clarifying question

Do NOT output questions to stdout — embed them in the spec files only.

User message:

{message}"#
                )
            } else {
                format!(
                    r#"You are a requirements integrator. Create a new spec knowledge base under the spec/ directory based on the following user message.

RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive knowledge base, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Exercise judgment about the weight and nature of each input. Not all inputs are equal — some are core requirements, others are asides or loosely structured thoughts. Summarize, condense, or reframe as appropriate to maintain coherence and quality, while always preserving intent.
- Integrate autonomously — do not ask the user to approve the output. If something is wrong, the user will submit corrective input.

SPEC STRUCTURE:
- Create spec/README.md as the primary entrypoint
- For a small initial spec, README.md should contain all the substance
- Use prose and lists only — no diagrams, tables, or non-textual content
- Stick to basic Markdown — headings, paragraphs, lists, bold/italic, links

CODEBASE CONTEXT:
You have read access to the project where this tool is running. Gather whatever codebase context you need to make sense of the user's requirements — look at relevant files, understand the domain, terminology, and existing structure. Do this autonomously without requiring user guidance.

INLINE QUESTIONS:
Embed clarifying questions directly in spec files using this format (one per line):
?Q{{id}}: {{question text}}

Place each question near the content it relates to. Assign sequential IDs starting from 1. Generate up to 9 questions focusing on the most important things to clarify. Each question should be self-contained — understandable without cross-referencing.

Do NOT output questions to stdout — embed them in the spec files only.

User message:

{message}"#
                )
            };

            // Run command while monitoring for new submissions
            let command_future = run_command(&config, &prompt);
            tokio::pin!(command_future);

            let result = loop {
                tokio::select! {
                    result = &mut command_future => break result,
                    msg = rx.recv() => {
                        if let Some(msg) = msg {
                            queue.push(msg);
                            let waiting = queue.len() - i - 1;
                            let status = format!("Integrating ({} in queue)...", waiting);
                            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(status));
                        }
                    }
                }
            };

            match result {
                Ok(_) => {
                    let questions = scan_inline_questions(&spec_dir);
                    let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(questions));
                }
                Err(e) => {
                    // Discard all remaining queued items
                    while let Ok(_) = rx.try_recv() {}
                    let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!("Error: {e}")));
                    errored = true;
                    break;
                }
            }

            i += 1;
        }

        if !errored {
            let _ = ui_tx.send(IntegratorMessage::IntegrationComplete);
        }
    }
}


async fn run_command(config: &IntegratorConfig, prompt: &str) -> Result<String, String> {
    let output = Command::new(&config.command)
        .args(&config.args)
        .arg(prompt)
        .current_dir(&config.working_dir)
        .env("CLAUDE_CODE_SIMPLE", "1")
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Failed to run {}: {e}", config.command))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} exited with error: {stderr}", config.command));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Scan all markdown files under spec/ for inline questions in the format:
/// ?Q{id}: {question text}
pub fn scan_inline_questions(spec_dir: &Path) -> Vec<(usize, String)> {
    let mut questions = Vec::new();
    if !spec_dir.exists() {
        return questions;
    }
    scan_dir_for_questions(spec_dir, &mut questions);
    questions.sort_by_key(|(id, _)| *id);
    questions
}

fn scan_dir_for_questions(dir: &Path, questions: &mut Vec<(usize, String)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_questions(&path, questions);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    if let Some(q) = parse_inline_question(line) {
                        questions.push(q);
                    }
                }
            }
        }
    }
}

fn parse_inline_question(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("?Q")?;
    let colon_pos = rest.find(':')?;
    let id: usize = rest[..colon_pos].parse().ok()?;
    let text = rest[colon_pos + 1..].trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some((id, text))
}
