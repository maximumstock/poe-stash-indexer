tidy:
	cargo fmt --all && cargo clippy --fix --all-features --all-targets -- -D warnings --no-deps

test:
	cargo test --all-features -- --nocapture