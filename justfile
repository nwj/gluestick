# just manual: https://just.systems/man/en/

_default:
	@just --list

# Audits the app's dependencies for security vulnerabilities and unpermitted licenses
audit:
	cargo deny check advisories && cargo deny check licenses

# Lints the codebase (via clippy)
check:
	cargo clippy --locked

# Formats the codebase (via cargo fmt)
format:
	cargo fmt && npx prettier --write "**/*.{html,css}" --log-level silent

# Builds and starts the app
run:
	cargo run --locked

# Runs all tests
test:
	cargo test --locked

# Lints, tests, builds, and runs the app on every change
watch:
	cargo watch -i "*.css" -c -x "clippy --locked" -x "test --locked" -x "run --locked"

# Lints, builds, and runs the app on every change. Useful where running the test suite slows iteration down too much, e.g. when writing html
watch-testless:
	cargo watch -i "*.css" -c -x "clippy --locked" -x "run --locked"
