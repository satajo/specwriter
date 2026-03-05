# specwriter

A terminal tool for turning a stream of thoughts into a cohesive spec. You type requirements, it integrates them into a single `SPEC.md` and asks clarifying questions — powered by Claude.

## How it works

1. Type your thoughts into the input box
2. Press `Ctrl+S` to submit
3. Background process calls `claude` CLI to integrate your message into `SPEC.md`
4. Open questions appear at the top to guide your thinking

That's it. Keep writing, keep submitting. The spec evolves with you.

## Install

```bash
nix run github:your-user/specwriter
```

Or from the repo:

```bash
nix build
./result/bin/specwriter
```

Requires `claude` CLI on your `$PATH`.

## Dev

```bash
nix develop
cargo build
cargo test --package specwriter-bdd  # BDD tests
```

## Keys

| Key | Action |
|-----|--------|
| `Ctrl+S` | Submit input |
| `Ctrl+C` | Quit |
| `Enter` | Newline |
