# Specwriter

Specwriter enables users to conveniently integrate requirement additions, updates,
and deletions into a project knowledge base. The user types requirements as a stream
of thoughts into a terminal UI, and each submission is integrated into a cohesive spec
that lives alongside the project.

## Core concept

The spec files in a project are the **data plane** — a declarative description of what
the project should be like, consumed downstream by whatever tools or people doing the
work. Specwriter is the **control plane** upstream of the specs: it's how the user
authors and maintains that declarative state.

The user doesn't need to organize their thoughts — they type what comes to mind, submit
it, and the system handles integration. Requirements can be added, refined, or retracted
over successive submissions. The spec evolves incrementally, always reflecting the
current state of the user's intent.

## How it works

- The user types into a TUI input area and presses Ctrl+S to submit
- Each submission triggers a background call to the Claude CLI, which reads the
  existing spec, interprets the user's message, and integrates it into the spec files
  under `spec/`
- `spec/README.md` is the primary entrypoint; additional files may be created as the
  knowledge base grows
- Clarifying questions are placed at the end of spec files under a `## Questions`
  heading (in `Q{id}: {text}` format) and surfaced in the UI to guide the user's
  thinking
- Multiple rapid submissions are queued and processed sequentially
- The UI shows integration status and queue depth
- Empty or whitespace-only input is ignored — submitting it does nothing
- The input area is cleared immediately after submission, freeing the user to type
  their next thought while the current one integrates
- The `spec/` directory is not created at launch — it comes into existence when the
  first submission is integrated

## UI layout

The screen shows four areas:
- **Status** — the current application state (see below)
- **Open Questions** — the current list of clarifying questions from the spec, sorted
  by priority from high to low, or "No open questions" if there are none. Each question
  displays its priority after the question number (e.g., "Q3 (p7): ...") and the name
  of the spec file it comes from, giving the user context for what area the question
  relates to.
- **Input** — a multiline text area where the user types, with a Ctrl+S submit hint
- **Help bar** — a single line at the bottom showing available keyboard shortcuts

## Keyboard shortcuts

- **Ctrl+S** — submit the current input for integration
- **Ctrl+C** — quit the application
- **Enter** — insert a newline (the input area supports multiline text)
- **Arrow keys** — move the cursor left/right within the text
- **Home / End** — move to the beginning/end of the current line
- **Backspace / Delete** — delete characters

## Status indicator

The status area reflects the application state with both text and a color-coded
indicator:
- **Ready** (green) — idle, waiting for input
- **Integrating** (yellow, animated spinner) — a submission is being processed; if
  additional submissions are queued, the display shows the queue depth (e.g.,
  "Integrating (1 in queue)..."). The queue count updates immediately when new
  submissions arrive, even while an integration is in progress.
- **Error** (red) — the last integration failed

Submitting new input from an error state recovers — the app transitions back to
"Integrating" and attempts the new submission.

## Startup behavior

When specwriter launches and finds existing spec files in the target directory, it
reads and displays any questions already embedded in them. It does **not** generate
new questions at startup — question generation only happens as part of integration
when the user submits input. If there are no existing spec files, the app starts in
the Ready state with no open questions.

## Error handling

- If the integrator command is not found or exits with an error, the UI shows "Error"
  with a red indicator
- When an error occurs, any remaining queued submissions are discarded
- The user can recover by simply submitting new input — no restart is needed

## Key properties

- **Write-path only**: Specwriter handles the write path — integrating user input
  into the spec. For reading and browsing the spec, any standard markdown or text file
  viewer works. There is no built-in spec viewer.
- **Stream-oriented**: The user doesn't edit the spec directly. They submit a stream
  of natural-language messages — additions, corrections, elaborations, deletions — and
  the integrator reconciles them into the knowledge base. The user may reference open
  questions by their IDs (e.g., "Q5: yes, single-user only") but this is optional —
  they can also address questions implicitly through natural-language input.
- **Furiously mutative**: The specwriter's core operation is aligning the spec to
  whatever the user says. What the user writes becomes the truth of specification —
  the spec is rewritten on-the-fly to conform. There is no history tracking,
  versioning, or change log; the spec simply reflects the current state of the user's
  intent. The integrator exercises judgment about how to incorporate each message — it
  may summarize, condense, restructure, or split content — but the user's input is
  authoritative. The user corrects by submitting further input, not by approving
  changes.
- **Self-organizing**: The specwriter autonomously manages the structure of the spec
  files — creating, splitting, merging, and renaming files as the knowledge base
  grows. There are no artificial limits on spec size or number of files.
  Self-organization is a core feature, not an incidental behavior.
- **Abstraction-preserving**: User input can arrive at any level of detail — from
  high-flying project goals and product vision down to specific technical choices and
  implementation details. The integrator's job is to appropriately integrate all of
  these levels, preserving each at the abstraction the user expressed it. It doesn't
  translate high-level ideas into implementation details, nor does it generalize
  specific technical decisions into vague principles.
- **Question-driven**: The system generates clarifying questions embedded in the spec
  to surface gaps, ambiguities, or contradictions. These help the user think through
  their requirements without requiring them to be exhaustive upfront. Questions are
  placed at the end of each spec file under a `## Questions` heading, formatted as
  `Q<number>: <question text>` with each question in its own paragraph. This keeps
  questions out of the way of the human reader while still being part of the spec
  files. Questions have stable numeric identifiers (Q1, Q2, ...) that persist across
  integrations. New questions continue from the highest existing ID. Answered or
  irrelevant questions are removed. The pool is capped at 9 questions to keep the list
  manageable. Each question is assigned a **priority** (1–9, where 1 = low and
  9 = high) so the user can focus on the most impactful questions first, enabling more
  efficient information gathering. Priority is based on two factors: how critical it
  is that this specific question gets answered, and how much new information about the
  spec would be gained from an answer.
- **Single-session**: Specwriter is designed for use within a single session. There is
  no built-in collaboration or multi-user support. However, since the spec files are
  plain Markdown, users can share them through normal means (e.g., committing to Git)
  if they choose.
- **Single-project scoped**: Specwriter operates in the current project directory.
  Multi-project workflows are out of scope — users open separate terminal sessions for
  different projects.
- **Claude Code backend**: The integration backend is Claude Code (the Claude CLI).
  The architecture doesn't need to be backend-agnostic, but shouldn't make it
  gratuitously hard to swap in another backend in the future.
- **Read-only project access**: When calling the underlying AI agent, the specwriter
  must ensure the agent has only read access to the project — no writes — with the
  sole exception of the `spec/` directory, where the agent must be allowed to write.
- **Token efficiency**: Specwriter should not consume unnecessarily many tokens. For
  the Claude Code CLI specifically, this means using a single session for the
  integrator — started on the first integration and resumed on subsequent ones using
  the appropriate CLI flags (e.g., `--resume` / `--session-id`). This avoids redundant
  context re-ingestion on every integration call.
- **No special import**: Specwriter doesn't have a dedicated import mechanism. When
  targeted at a directory that already contains spec files, it simply operates on
  them — reading the existing content and integrating new input as usual. On startup,
  any existing questions in the spec files are displayed, but no new questions are
  generated until the user submits input. Starting from scratch (empty `spec/`
  directory) is the default case.

## Formatting

Spec files should have line lengths limited to approximately 120 characters to remain
readable in terminals and when viewed in raw format.

## Packaging

Specwriter is packaged as a Nix flake that produces a `specwriter` binary.