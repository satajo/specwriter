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

    // Seed questions from existing spec
    if spec_path.exists() {
        let content = std::fs::read_to_string(&spec_path).unwrap_or_default();
        if !content.trim().is_empty() {
            let _ = ui_tx.send(IntegratorMessage::StatusUpdate(
                "Loading existing specs...".into(),
            ));

            let prompt = format!(
                r#"You are a requirements integrator. Read the existing SPEC.md file and generate an initial set of open questions based on its current content. These should be the most important things the user should think about or clarify.

Question rules:
- Questions must stay grounded in the topics and themes present in the spec
- Match the abstraction level of the spec content
- Generate up to 9 questions, sorted by priority

NEXT_QUESTION_ID: {next_question_id}

Assign each question an integer ID starting from NEXT_QUESTION_ID. Output the questions as a JSON array on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:[{{"id":1,"text":"Question 1?"}},{{"id":2,"text":"Question 2?"}}]"#
            );

            match run_command(&config, &prompt).await {
                Ok(output) => {
                    current_questions = parse_questions(&output);
                    if !current_questions.is_empty() {
                        next_question_id = current_questions
                            .iter()
                            .map(|(id, _)| *id)
                            .max()
                            .unwrap_or(0)
                            + 1;
                    }
                    let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(
                        current_questions.clone(),
                    ));
                }
                Err(e) => {
                    let _ = ui_tx.send(IntegratorMessage::StatusUpdate(format!("Error: {e}")));
                }
            }
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

            // Create empty SPEC.md if it doesn't exist (app owns file creation, not CLI)
            if !spec_path.exists() {
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
                    "\n\nCURRENT QUESTION POOL (manage incrementally — keep relevant, remove answered, add new):\n{}\n\nNEXT_QUESTION_ID: {}\n",
                    q_list.join("\n"),
                    next_question_id
                )
            };

            let pool_cap_note = if current_questions.len() >= 9 {
                "\n\nThe question pool is at capacity (9 questions). Do NOT add new questions. You may keep or remove existing ones only.\n"
            } else {
                ""
            };

            let question_instructions = format!(
                r#"QUESTIONS:
Update the question pool incrementally based on the current state of the spec:
- KEEP questions that are still relevant and unanswered (preserve their ID)
- REMOVE questions that have been answered or are no longer relevant
- ADD new questions that arise from the latest input (assign IDs starting from NEXT_QUESTION_ID, never reuse old IDs)
- You may slightly reword kept questions, but their ID must not change
- Questions must match the abstraction level of the spec
- Questions must stay grounded in the topics and themes present in the spec
- If you detect conflicting requirements, surface them as clarifying questions
- Maximum 9 questions in the pool{pool_cap_note}

{next_id_context}Output the updated question pool as a JSON array on the LAST line of your response, prefixed with "QUESTIONS:" like this:
QUESTIONS:[{{"id":1,"text":"Question 1?"}},{{"id":3,"text":"Updated question 3?"}},{{"id":5,"text":"New question?"}}]"#,
                next_id_context = if current_questions.is_empty() {
                    format!("NEXT_QUESTION_ID: {}\n\n", next_question_id)
                } else {
                    String::new() // already included in questions_context
                }
            );

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

{question_instructions}"#
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

{question_instructions}"#
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
                Ok(output) => {
                    current_questions = parse_questions(&output);
                    if !current_questions.is_empty() {
                        next_question_id = current_questions
                            .iter()
                            .map(|(id, _)| *id)
                            .max()
                            .unwrap_or(0)
                            + 1;
                    }
                    let _ = ui_tx.send(IntegratorMessage::QuestionsUpdated(
                        current_questions.clone(),
                    ));
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

#[derive(serde::Deserialize)]
struct QuestionEntry {
    id: usize,
    text: String,
}

pub fn parse_questions(output: &str) -> Vec<(usize, String)> {
    for line in output.lines().rev() {
        let line = line.trim();
        if let Some(json_str) = line.strip_prefix("QUESTIONS:") {
            // Try new format: [{"id":1,"text":"..."},...]
            if let Ok(questions) = serde_json::from_str::<Vec<QuestionEntry>>(json_str) {
                return questions.into_iter().map(|q| (q.id, q.text)).collect();
            }
            // Fallback: old format ["Q1","Q2"] — assign sequential IDs
            if let Ok(questions) = serde_json::from_str::<Vec<String>>(json_str) {
                return questions
                    .into_iter()
                    .enumerate()
                    .map(|(i, q)| (i + 1, q))
                    .collect();
            }
        }
    }
    Vec::new()
}
