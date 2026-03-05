use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum IntegratorMessage {
    QuestionsUpdated(Vec<String>),
    StatusUpdate(String),
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
    let mut pending: Vec<String> = Vec::new();

    loop {
        let msg = match rx.recv().await {
            Some(m) => m,
            None => return,
        };
        pending.push(msg);

        while let Ok(msg) = rx.try_recv() {
            pending.push(msg);
        }

        let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
            "Integrating...".into(),
        ));

        let has_spec = spec_path.exists();
        let messages_text = pending
            .iter()
            .enumerate()
            .map(|(i, m)| format!("Message {}:\n{}", i + 1, m))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let prompt = if has_spec {
            format!(
                r#"You are a requirements integrator. Read the existing SPEC.md file, then integrate the following new user messages into it. The goal is to maintain a single cohesive requirements/feature specification document.

IMPORTANT RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive document, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Keep the document well-organized with clear sections.

New messages to integrate:

{messages_text}

After reading the existing SPEC.md, rewrite it with the new messages integrated. Write the updated content to SPEC.md.

Then, think of up to 3 open questions that would help clarify or improve the spec. These should be things the user should think about or clarify. Output the questions as a JSON array of strings on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:["Question 1?","Question 2?","Question 3?"]"#
            )
        } else {
            format!(
                r#"You are a requirements integrator. Create a new SPEC.md file based on the following user messages. The goal is to create a cohesive requirements/feature specification document.

IMPORTANT RULES:
- Match the user's level of abstraction. Do NOT translate their inputs into technical implementation details unless they are already at that level.
- You are integrating a thought-stream of requirements into a cohesive document, not writing a technical spec.
- Preserve the user's intent and language where possible.
- Keep the document well-organized with clear sections.

User messages:

{messages_text}

Create a SPEC.md file with these requirements integrated into a cohesive document.

Then, think of up to 3 open questions that would help clarify or improve the spec. These should be things the user should think about or clarify. Output the questions as a JSON array of strings on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:["Question 1?","Question 2?","Question 3?"]"#
            )
        };

        match run_command(&config, &prompt).await {
            Ok(output) => {
                pending.clear();
                let questions = parse_questions(&output);
                let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(questions));
                let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
                    "Integration complete. SPEC.md updated.".into(),
                ));
            }
            Err(e) => {
                let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
                    format!("Error: {e}"),
                ));
            }
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
