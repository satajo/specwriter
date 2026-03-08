# specwriter

A terminal UI for building structured requirement specs through conversational refinement with Claude.

## Quick start

```bash
nix run github:satajo/specwriter
```

Requires `claude` CLI on PATH.

## Installation

Install to your user profile:

```bash
nix profile install github:satajo/specwriter
```

Or add to your NixOS/home-manager configuration:

```nix
{
  inputs.specwriter.url = "github:satajo/specwriter";
}
```

Then include `inputs.specwriter.packages.${system}.default` in `environment.systemPackages` or your home-manager `home.packages`.

Requires the `claude` CLI to be available on `$PATH`.

## Usage

Run `specwriter` in your project directory. Type your requirements into the input box and press Ctrl+S to submit. The spec builds incrementally as you go.

## Features

- **Core workflow** -- type requirements, submit with Ctrl+S, background integration into SPEC.md via Claude CLI
- **Tabbed UI** -- Writer, Open Questions, Spec Viewer, and Settings tabs (Tab/Shift+Tab)
- **Clarifying questions** -- auto-generated, priority-sorted (1--5), browsable with arrow keys
- **Answer dialog** -- answer questions inline; pick from suggested solutions or write a custom answer
- **Spec viewer** -- read SPEC.md inside the TUI with scrolling
- **Message queue** -- submit while integration is in progress; queue count in status bar
- **Configurable settings** -- Claude command, model, spec filename, WebSearch/WebFetch toggles
- **Session recovery** -- transparent recovery from Claude CLI session errors
- **Quit confirmation** -- Ctrl+C twice to force-quit during active integration
- **Startup seeding** -- loads existing questions from SPEC.md on launch

## Keybindings

| Key | Action |
|-----|--------|
| Ctrl+S | Submit |
| Ctrl+C | Quit |
| Tab | Next tab |

The app shows context-specific keybinding hints at the bottom of the screen.

## Building from source

Enter the dev shell:

```bash
nix develop
```

Then build and test:

```bash
cargo build --workspace
cargo test --package specwriter-tui-testdriver
```

See [CLAUDE.md](CLAUDE.md) for the full development workflow.

## License

Copyright (c) Sami Jokela.

Specwriter is licensed under the [GNU General Public License v3.0](COPYING) or later.
