use std::path::PathBuf;
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
                "Edit,Read".into(),
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
    let spec_path = config.working_dir.join("SPEC.md");
    let mut current_questions: Vec<(usize, String)> = Vec::new();
    let mut next_question_id: usize = 1;

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
            // Check for new messages before each step
            while let Ok(msg) = rx.try_recv() {
                queue.push(msg);
            }

            let total = queue.len();
            let status = if total >= 2 {
                format!("Integrating {}/{}...", i + 1, total)
            } else {
                "Integrating...".into()
            };
            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(status));

            // Create empty SPEC.md if it doesn't exist (app owns file creation, not CLI)
            let has_spec = spec_path.exists();
            if !has_spec {
                if let Err(e) = std::fs::write(&spec_path, "") {
                    let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!(
                        "Error: Failed to create SPEC.md: {e}"
                    )));
                    errored = true;
                    break;
                }
            }
            let spec_is_empty = std::fs::read_to_string(&spec_path)
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);
            let message = &queue[i];

            let questions_context = if current_questions.is_empty() {
                String::new()
            } else {
                let q_list: Vec<String> = current_questions
                    .iter()
                    .map(|(num, q)| format!("Q{}. {}", num, q))
                    .collect();
                format!(
                    "\n\nOPEN QUESTIONS (currently pending — do not re-ask these):\n{}\n",
                    q_list.join("\n")
                )
            };

            let prompt = if !spec_is_empty {
                format!(
                    r#"You are a requirements integrator. Read the existing SPEC.md file, then integrate the following new user message into it. The goal is to maintain a single cohesive requirements/feature specification document.

RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive document, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Exercise judgment about the weight and nature of each input. Not all inputs are equal — some are core requirements, others are asides or loosely structured thoughts. Summarize, condense, or reframe as appropriate to maintain document coherence and quality, while always preserving intent.
- If the input seems unrelated to existing content, do not ignore it. The user may be switching topics or expanding scope. Create a new topic area, weave it into existing sections, or restructure the document as needed.
- Integrate autonomously — do not ask the user to approve the output. If something is wrong, the user will submit corrective input.

DOCUMENT STRUCTURE:
The spec should include:
- An overview of the project and what it does
- Requirements organized by topic area
- Acceptance criteria that capture what "done" looks like for key requirements
Let this structure evolve naturally as content is added — organize into these sections as it makes sense rather than forcing a rigid template.

CODEBASE CONTEXT:
You have read access to the project where this tool is running. Gather whatever codebase context you need to make sense of the user's requirements — look at relevant files, understand the domain, terminology, and existing structure. Do this autonomously without requiring user guidance. The depth of exploration should match the specificity of the user's input: specific features warrant targeted context, general input warrants broader exploration.

User message:

{message}{questions_context}

After reading the existing SPEC.md, rewrite it with the new message integrated. Write the updated content to SPEC.md.

QUESTIONS:
Generate a fresh set of open questions based on the CURRENT state of the spec. These should be the most important things the user should think about or clarify, sorted by priority (most important first). Do NOT carry over old questions — generate them anew from the spec.

Question rules:
- If you detect conflicting or contradictory requirements, surface them as clarifying questions. Do not silently resolve or ignore conflicts.
- Questions must stay grounded in the topics and themes present in the spec. Do not drift into tangential areas or introduce concerns the user hasn't raised. Questions should feel like natural follow-ups to what's already described.

Output the questions as a JSON array of strings on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:["Question 1?","Question 2?","Question 3?"]"#
                )
            } else {
                format!(
                    r#"You are a requirements integrator. Create a new SPEC.md file based on the following user message. The goal is to create a cohesive requirements/feature specification document.

RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive document, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Exercise judgment about the weight and nature of each input. Not all inputs are equal — some are core requirements, others are asides or loosely structured thoughts. Summarize, condense, or reframe as appropriate to maintain document coherence and quality, while always preserving intent.
- Integrate autonomously — do not ask the user to approve the output. If something is wrong, the user will submit corrective input.

DOCUMENT STRUCTURE:
The spec should include:
- An overview of the project and what it does
- Requirements organized by topic area
- Acceptance criteria that capture what "done" looks like for key requirements
Let this structure evolve naturally as content is added — organize into these sections as it makes sense rather than forcing a rigid template.

CODEBASE CONTEXT:
You have read access to the project where this tool is running. Gather whatever codebase context you need to make sense of the user's requirements — look at relevant files, understand the domain, terminology, and existing structure. Do this autonomously without requiring user guidance. The depth of exploration should match the specificity of the user's input: specific features warrant targeted context, general input warrants broader exploration.

User message:

{message}{questions_context}

Create a SPEC.md file with this requirement integrated into a cohesive document.

QUESTIONS:
Generate a fresh set of open questions based on the CURRENT state of the spec. These should be the most important things the user should think about or clarify, sorted by priority (most important first).

Question rules:
- If you detect conflicting or contradictory requirements, surface them as clarifying questions. Do not silently resolve or ignore conflicts.
- Questions must stay grounded in the topics and themes present in the spec. Do not drift into tangential areas or introduce concerns the user hasn't raised. Questions should feel like natural follow-ups to what's already described.

Output the questions as a JSON array of strings on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:["Question 1?","Question 2?","Question 3?"]"#
                )
            };

            match run_command(&config, &prompt).await {
                Ok(output) => {
                    let raw_questions = parse_questions(&output);
                    current_questions = raw_questions
                        .into_iter()
                        .enumerate()
                        .map(|(i, q)| (next_question_id + i, q))
                        .collect();
                    next_question_id += current_questions.len();
                    let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(current_questions.clone()));
                }
                Err(e) => {
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

pub fn parse_questions(output: &str) -> Vec<String> {
    for line in output.lines().rev() {
        let line = line.trim();
        if let Some(json_str) = line.strip_prefix("QUESTIONS:") {
            if let Ok(questions) = serde_json::from_str::<Vec<String>>(json_str) {
                return questions;
            }
        }
    }
    Vec::new()
}
