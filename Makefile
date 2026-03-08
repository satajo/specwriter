check:
	cargo fmt --check
	cargo build --workspace
	cargo clippy --workspace -- -D warnings
	cargo test --package specwriter-tui-testdriver
