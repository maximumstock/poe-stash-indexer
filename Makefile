tidy:
	cargo fmt --all && cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets -- -D warnings --no-deps

test:
	cargo test --all-features -- --nocapture
