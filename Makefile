tidy:
	cargo fmt --all && cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets -- -D warnings

tidy-check:
	cargo fmt --check && cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test --all-features -- --nocapture
