# specwriter

Rust TUI app (ratatui + crossterm + tokio) for integrating requirements into a SPEC.md via Claude CLI.

## Project structure

Cargo workspace with two members:

- `specwriter/` — the app (lib + binary)
  - `src/lib.rs` — App state, handle_key, AppRunner (test harness)
  - `src/main.rs` — crossterm event loop, thin wrapper over lib
  - `src/ui.rs` — ratatui rendering (status, questions, input, help)
  - `src/integrator.rs` — background tokio task, calls claude CLI, parses questions
- `tui-testdriver/` — Cucumber BDD test suite
  - `tests/tui_testdriver.rs` — step definitions using AppRunner
  - `mock-claude*.sh` — mock scripts replacing claude CLI in tests
- `features/` — Gherkin specs (integration, questions, error_handling, etc.)

## Key patterns

- `IntegratorConfig` makes the claude command injectable (tests swap in mock scripts)
- `AppRunner` drives the full TUI through ratatui's `TestBackend` — BDD tests are true end-to-end
- `handle_key` in lib.rs is shared between main.rs and AppRunner — single code path
- NixOS: shebangs must use `#!/usr/bin/env bash` (no /bin/bash)
- `nix develop` required for cargo — no system rust toolchain

## Building & testing

```
nix develop --command make check
```

## Development workflow

**All features and spec changes MUST follow BDD-first TDD:**

1. **Spec first** — Write or update `.feature` files in `features/` before touching any Rust code. The Gherkin scenarios define what the feature does in implementation-agnostic terms.
2. **Red** — Run the tests, confirm the new scenarios fail.
3. **Green** — Implement the minimum Rust code to make them pass.
4. **Refactor** — Clean up if needed, ensure all tests still pass.
5. **Docs** — Update CLAUDE.md and README.md if the change affects project structure, patterns, or usage.

This applies to any task that adds, changes, or removes user-facing behavior. The feature suite is the source of truth for what specwriter does — no important behavior should exist without a corresponding scenario.

## Writing Gherkin

Read `GHERKIN_GUIDE.md` before writing or modifying feature files. Key points: behavior over mechanics, declarative steps, one behavior per scenario, domain language, 3-7 steps per scenario.

## Conventions

- No Python. Rust only.
- BDD tests are the primary test suite — no brittle unit tests
- Feature files serve as living documentation and specification
- Every user-facing feature must have Gherkin coverage before implementation begins
- Keep it simple. Don't over-engineer.
