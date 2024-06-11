# just manual: https://just.systems/man/en/

_default:
	@just --list

# Lints the codebase (via clippy)
check:
	cargo clippy --locked

# Runs all tests
test:
	cargo test --locked

# Builds and starts the app
run:
	cargo run --locked

# Lints, tests, builds, and runs the app on every change
watch:
	cargo watch -i "*.css" -x "clippy --locked" -x "test --locked" -x "run --locked"

# Formats the codebase (via cargo fmt)
format:
	cargo fmt

# Audits the app's dependencies for security vulnerabilities and unpermitted licenses
audit:
	cargo deny check advisories && cargo deny check licenses
