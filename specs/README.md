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
  under `specs/`
- `specs/README.md` is the primary entrypoint; additional files may be created as the
  knowledge base grows
- Clarifying questions are placed at the end of spec files under a `## Questions`
  heading, each as a `###` subheading (e.g., `### Q3 (p7): Short title`) with the
  question body as prose underneath. They are surfaced in the UI to guide the user's
  thinking
- Multiple rapid submissions are queued and processed sequentially
- The UI shows integration status and queue depth
- Empty or whitespace-only input is ignored — submitting it does nothing
- The input area is cleared immediately after submission, freeing the user to type
  their next thought while the current one integrates
- The `specs/` directory is not created at launch — it comes into existence when the
  first submission is integrated

## UI layout

The UI is modal, organized into two tabs:
- **Text Input** (green) — free-form text field for typing requirements, same as the
  current input experience
- **Open Questions (N)** (blue) — browsable list of clarifying questions with
  answer-in-place functionality; the tab name shows the total number of open questions

Tab titles use Title Case — each word starts with a capital letter.
Tab content views should not have their own titles — the tab title is sufficient. Avoid
redundancy between the tab label and any heading or border title on the view it opens.

All text input areas (the main input, the answer dialog, etc.) should have 1 character of
left padding so text doesn't sit flush against the border.

The tab bar appears below the status area. The screen shows these areas top to bottom:
- **Status** — the current application state (see below); displayed as plain text
  with no border or box around it; no extra left padding — it aligns with the panel
  borders below it
- *(one empty line of spacing)*
- **Tab bar** — shows the two tabs with their respective colors; the active tab is
  highlighted by inverting its foreground and background colors
- **Tab content** — the active tab's content area (see below)
- **Help bar** — a single line at the bottom showing available keyboard shortcuts

### Text Input tab

A multiline text area where the user types whatever they want, with a Ctrl+S submit
hint. This is the primary input mode — the user writes free-form requirements,
corrections, or responses and submits them for integration.

### Open Questions tab

A list of clarifying questions from the spec, sorted by priority from high to low.
Each question displays its priority after the question number (e.g., "Q3 (p7): ...")
and the name of the spec file it comes from, giving the user context for what area
the question relates to.

The user browses questions with arrow keys (up/down). The question list supports
scrolling when the number of questions exceeds the visible area. The focused question
is highlighted, and its full content is shown in a separate panel below the list.

Pressing Enter on a focused question opens a dialog where the user can type an answer.
On submission, the answer is sent to the integrator with the relevant context (e.g.,
"The answer to question Q3 is: ..."). This gives the user a direct, structured way to
answer questions without having to reference question IDs in free-form text.

If there are no open questions, the tab shows "No open questions."

## Keyboard shortcuts

### Global

- **Ctrl+C** — quit the application
- **Tab** — switch between Text Input and Open Questions tabs

### Text Input tab

- **Ctrl+S** — submit the current input for integration
- **Enter** — insert a newline (the input area supports multiline text)
- **Arrow keys** — move the cursor left/right within the text
- **Home / End** — move to the beginning/end of the current line
- **Backspace / Delete** — delete characters

### Open Questions tab

- **Up / Down** — move focus between questions
- **Enter** — open the answer dialog for the focused question

### Answer dialog

- **Ctrl+S** — submit the answer
- **Esc** — cancel and close the dialog
- **Enter** — insert a newline
- Standard text editing keys (arrows, Home/End, Backspace/Delete)

## Status indicator

The status area reflects the application state with both text and a color-coded
indicator:
- **Ready** (green) — displays "Idle." with no additional instructional text
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
  the integrator reconciles them into the knowledge base. Questions can be answered
  either through the dedicated Open Questions tab (which provides structured
  answer-in-place functionality) or implicitly through free-form text input.
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
  placed at the end of each spec file under a `## Questions` heading. Each question
  is a `###` subheading with the format `### Q<number> (p<priority>): <title>`,
  followed by the question body as prose underneath. This structure supports
  multi-line questions naturally — the title gives a scannable summary, and the body
  can elaborate as needed. The parser collects all body text until the next `###`
  heading or end of file. This keeps questions out of the way of the human reader
  while still being part of the spec files. Questions have stable numeric identifiers
  (Q1, Q2, ...) that persist across integrations. New questions continue from the
  highest existing ID. Answered or irrelevant questions are removed. There is no
  artificial cap on the number of questions — the dedicated scrollable list view
  in the Open Questions tab handles any number. Each question is assigned a
  **priority** (1–9, where 1 = low and 9 = high) so the user can focus on the most
  impactful questions first, enabling more efficient information gathering. Priority
  is based on two factors: how critical it is that this specific question gets
  answered, and how much new information about the spec would be gained from an
  answer.
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
  sole exception of the `specs/` directory, where the agent must be allowed to write.
- **Token efficiency**: Specwriter should not consume unnecessarily many tokens. For
  the Claude Code CLI specifically, this means using a single session for the
  integrator — started on the first integration and resumed on subsequent ones using
  the appropriate CLI flags (e.g., `--resume` / `--session-id`). This avoids redundant
  context re-ingestion on every integration call.
- **No special import**: Specwriter doesn't have a dedicated import mechanism. When
  targeted at a directory that already contains spec files, it simply operates on
  them — reading the existing content and integrating new input as usual. On startup,
  any existing questions in the spec files are displayed, but no new questions are
  generated until the user submits input. Starting from scratch (empty `specs/`
  directory) is the default case.

## Formatting

Spec files should have line lengths limited to approximately 120 characters to remain
readable in terminals and when viewed in raw format.

## Packaging

Specwriter is packaged as a Nix flake that produces a `specwriter` binary.

## Questions

### Q1 (p6): Scrolling in other UI panels

The question list supports scrolling, but should the question detail panel, the text input area,
and the answer dialog also scroll when content exceeds the visible area?

### Q2 (p6): Quit-during-integration behavior

When the user quits with Ctrl+C while an integration is in progress, should the app wait for the
current integration to finish, kill it immediately, or offer a choice?

### Q4 (p5): Session expiry handling

The spec says the integrator uses `--session-id` and `--resume` for token efficiency. What should
happen if the Claude CLI session expires or becomes invalid mid-session? Should specwriter detect
this and start a fresh session, or surface it as an error?

### Q5 (p4): Target directory configuration

Should specwriter accept a target directory as a command-line argument, or does it always operate
in the current working directory? The spec says "current project directory" but doesn't specify
how that's determined.

### Q6 (p4): Agent read access scope

The implementation passes `--allowedTools Edit,Read,Write` to the Claude CLI, but the `Read` tool
gives the agent read access to the entire filesystem, not just the project. Is this intentional,
or should the agent's read access be scoped to the project directory?