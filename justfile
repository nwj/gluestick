# just manual: https://just.systems/man/en/

_default:
	@just --list

# Lints the codebace (via Clippy)
check:
	cargo clippy --locked -- -D warnings

# Runs all unit tests
test:
	cargo test --locked

# Builds and starts the app
run:
	cargo run --locked

# Lints, tests, builds, and runs the app on every change
watch:
	cargo watch -i "*.css" -x "clippy --locked -- -D warnings" -x "test --locked" -x "run --locked"
