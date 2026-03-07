use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum IntegratorMessage {
    QuestionsUpdated(Vec<String>),
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
                "Edit,Write,Read".into(),
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
    let mut current_questions: Vec<String> = Vec::new();

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

            let has_spec = spec_path.exists();
            let message = &queue[i];

            let questions_context = if current_questions.is_empty() {
                String::new()
            } else {
                let q_list: Vec<String> = current_questions
                    .iter()
                    .enumerate()
                    .map(|(i, q)| format!("{}. {}", i + 1, q))
                    .collect();
                format!(
                    "\n\nOPEN QUESTIONS (currently pending — do not re-ask these):\n{}\n",
                    q_list.join("\n")
                )
            };

            let prompt = if has_spec {
                format!(
                    r#"You are a requirements integrator. Read the existing SPEC.md file, then integrate the following new user message into it. The goal is to maintain a single cohesive requirements/feature specification document.

IMPORTANT RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive document, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Keep the document well-organized with clear sections.

User message:

{message}{questions_context}

After reading the existing SPEC.md, rewrite it with the new message integrated. Write the updated content to SPEC.md.

Then, generate a fresh set of open questions based on the CURRENT state of the spec. These should be the most important things the user should think about or clarify, sorted by priority (most important first). Do NOT carry over old questions — generate them anew from the spec. Output the questions as a JSON array of strings on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:["Question 1?","Question 2?","Question 3?"]"#
                )
            } else {
                format!(
                    r#"You are a requirements integrator. Create a new SPEC.md file based on the following user message. The goal is to create a cohesive requirements/feature specification document.

IMPORTANT RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive document, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Keep the document well-organized with clear sections.

User message:

{message}{questions_context}

Create a SPEC.md file with this requirement integrated into a cohesive document.

Then, generate a fresh set of open questions based on the CURRENT state of the spec. These should be the most important things the user should think about or clarify, sorted by priority (most important first). Output the questions as a JSON array of strings on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:["Question 1?","Question 2?","Question 3?"]"#
                )
            };

            match run_command(&config, &prompt).await {
                Ok(output) => {
                    current_questions = parse_questions(&output);
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
