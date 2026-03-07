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

### Color philosophy

Because this is a terminal UI application, colors should be used intentionally. All text should use the
terminal's default foreground color unless an accent color is specifically called for. This ensures the
application respects the user's terminal theme and only uses color where it carries meaning — status
indicators, active tab highlights, focused-item markers, and similar purposeful accents.

### Tabs and layout

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

The tabs should use the ratatui `Tabs` widget. The tab bar itself is a standalone
row with no borders. Each tab's content is responsible for drawing its own borders —
the tab bar does not provide a shared bordered container. This means different tabs
can have different visual structures (e.g., the Text Input tab draws a single bordered
box, while the Open Questions tab draws two separate bordered boxes for the list and
detail panels).

The screen shows these areas top to bottom:
- **Status** — the current application state (see below); displayed as plain text
  with no border or box around it; no extra left padding — it aligns with the panel
  borders below it
- *(one empty line of spacing)*
- **Tab bar + content** — the ratatui `Tabs` component renders the tab bar as a
  standalone row; the content area below it is unbordered at the tab level — each
  tab's content draws its own borders as needed; the active tab is highlighted by
  inverting its foreground and background colors
- **Help bar** — a single line at the bottom showing available keyboard shortcuts;
  the help bar is always context-sensitive, adapting its content to the current mode.
  This includes the Text Input tab, Open Questions tab, answer dialog, and quit
  confirmation dialog — each shows its own relevant shortcuts

### Scrolling

All navigable panels — text inputs, question list, question detail, answer dialog —
support scrolling when content exceeds the visible area. Scrolling uses
**center-focused** behavior: the focused item (cursor line or selected entry) stays
roughly in the middle of the viewport, except when near the beginning or end of the
content, where the view pins to the top or bottom respectively.

### Text Input tab

A multiline text area where the user types whatever they want, with a Ctrl+S submit
hint. This is the primary input mode — the user writes free-form requirements,
corrections, or responses and submits them for integration. All textarea inputs
(main input, answer dialog, etc.) must correctly handle long lines that soft-wrap
at the edge of the widget — the caret position must remain accurate after wrapping.

When the text input is empty, it displays placeholder text in a dimmed color:
"Type your requirements here. Ctrl+S to submit."

### Open Questions tab

A list of clarifying questions from the spec, sorted by priority from high to low.
Each question displays its priority after the question number (e.g., "Q3 (p7): ...")
and the name of the spec file it comes from, giving the user context for what area
the question relates to. When the question list is refreshed after an integration,
the UI preserves focus on the same question (matched by question ID). If the
previously focused question no longer exists, focus falls back using the same rules
as after answering: next question, then previous, then empty state.

The user browses questions with arrow keys (up/down). The focused question is
highlighted, and its full content is shown in a separate detail box below the
question list. The question list and the detail box are visually distinct — each
draws its own bordered box. The tab container itself should not draw borders;
instead, the content panels (question list and question detail) each draw their
own borders. This means the detail box is "outside" the tab's visual container,
appearing as a peer-level box rather than a sub-panel nested inside the list. The
vertical space is split 50/50 between the question list and the detail box. The
detail box is only shown when a question is selected — if there are no questions,
only the list box is displayed (taking the full content area).

Pressing Enter on a focused question opens a dialog where the user can type an answer.
On submission, the answer is sent to the integrator with the relevant context (e.g.,
"The answer to question Q3 is: ..."). The answered question is immediately removed from
the open questions list in the UI, giving the user clear feedback that their answer went
through. After removal, focus moves to the next question in the list (the one that was
below the answered question). The rationale is that if the user scrolled past earlier
questions, they weren't interested in answering those right now. If there is no next
question, focus moves to the previous one. If the list is now empty, the tab shows the
empty-list state. This gives the user a direct, structured way to answer questions
without having to reference question IDs in free-form text.

If there are no open questions, the tab shows a full-height bordered list box
containing "No open questions." — no detail box is shown.

## Keyboard shortcuts

### Global

- **Ctrl+C** — quit the application. If the app is idle, it quits immediately. If
  an integration is in progress, a confirmation pop-up dialog is shown (similar in
  style to the answer dialog) with the border title "Confirm Quit" and body text
  "Integration in progress. Press Ctrl+C again to quit." The user presses Ctrl+C
  again to confirm, or Esc to dismiss the dialog and cancel the quit. On confirmed
  exit, any in-progress integration is stopped immediately and any queued submissions
  are silently discarded — nothing continues running in the background
- **Tab** — switch between Text Input and Open Questions tabs

When a pop-up dialog is open (answer dialog, quit confirmation, etc.), it captures
all input — global shortcuts like Tab are ignored. The dialog must be explicitly
dismissed (via Esc or its own submit/confirm action) before normal navigation resumes.

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

The answer dialog also displays placeholder text in a dimmed color when empty:
"Type your answer here. Ctrl+S to submit."

- **Ctrl+S** — submit the answer; empty or whitespace-only input is ignored (same
  as the main text input)
- **Esc** — cancel and close the dialog
- **Enter** — insert a newline
- Standard text editing keys (arrows, Home/End, Backspace/Delete)

## Status indicator

The entire status line is colored to reflect the current application state — the
color applies to all text on the line, not just an icon or indicator element:
- **Ready** (default terminal foreground) — displays "Idle." with no additional
  instructional text; uses the terminal's normal foreground color, indicating
  nothing noteworthy is happening
- **Integrating** (yellow) — a submission is being processed. The animation is a
  trailing dot sequence (`Integrating.`, `Integrating..`, `Integrating...`) — the
  dots are always at the end of the line, cycling at 300ms per frame (fast enough
  to convey active work rather than lag). When additional submissions are queued,
  the queue depth appears before the dots (e.g., `Integrating (1 in queue)...`).
  The queue count updates immediately when new submissions arrive, even while an
  integration is in progress.
- **Error** (red) — the last integration failed; displays `Error! <description>`
  where the description is a one-line summary of what went wrong

Submitting new input from an error state recovers — the app transitions back to
"Integrating" and attempts the new submission.

## Startup behavior

When specwriter launches and finds existing spec files in the target directory, it
reads and displays any questions already embedded in them. It does **not** generate
new questions at startup — question generation only happens as part of integration
when the user submits input. If there are no existing spec files, the app starts in
the Ready state with no open questions.

## Error handling

- If the integrator command is not found or exits with an error, the status line
  shows `Error! <description>` in red. The description is the first line of the
  CLI's stderr output. If stderr is empty, the fallback is "Exit code N with no
  message" (where N is the process exit code). If neither an exit code nor stderr
  is available, the description is "Unknown reason".
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
- **Question-driven**: The system proactively generates clarifying questions embedded
  in the spec to surface gaps, ambiguities, or contradictions. These help the user
  think through their requirements without requiring them to be exhaustive upfront.
  The integrator should be aggressive about generating questions — there should always
  be at least a few open questions after each integration. A spec with zero questions
  is a sign the integrator isn't doing its job: every spec has unexplored dimensions,
  unstated assumptions, or areas that could benefit from clarification. Questions are
  the primary mechanism for driving the conversation forward. Questions are
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
- **Single-project scoped**: Specwriter operates in the current working directory.
  The specs directory defaults to `specs/` but can be overridden via a command-line
  argument (e.g., to use a different directory name). This only controls the name of
  the specs directory — it does not change the working directory or project root.
  Multi-project workflows are out of scope — users open separate terminal sessions for
  different projects.
- **Claude Code backend**: The integration backend is Claude Code (the Claude CLI).
  The architecture doesn't need to be backend-agnostic, but shouldn't make it
  gratuitously hard to swap in another backend in the future.
- **Read-only project access**: When calling the underlying AI agent, the specwriter
  must ensure the agent has only read access to the current project directory — not
  the entire filesystem — with the sole exception of the specs directory, where the
  agent must be allowed to write.
- **Token efficiency**: Specwriter should not consume unnecessarily many tokens. For
  the Claude Code CLI specifically, this means using a single session for the
  integrator — started on the first integration and resumed on subsequent ones using
  the appropriate CLI flags (e.g., `--resume` / `--session-id`). This avoids redundant
  context re-ingestion on every integration call. If the session expires or becomes
  invalid mid-session, specwriter should self-recover by creating a new session
  transparently — no user intervention required.
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

### Q22 (p3): Ctrl+C inside answer dialog

When the answer dialog is open and the user presses Ctrl+C, should it close the dialog (treating it like
Esc), quit the app, or do nothing? Since pop-up dialogs capture all input, Ctrl+C's normal quit behavior
would be blocked — but Ctrl+C might feel natural as a "cancel" action to some users.

### Q23 (p3): Question list scroll position after integration

When an integration completes and the question list is refreshed, the spec says focus is preserved by
question ID. Should the scroll position also be preserved (keeping the viewport roughly where it was), or
is it acceptable for the list to re-center on the focused item using the standard center-focused scrolling?

