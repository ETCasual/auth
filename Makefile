.PHONY: deps dev

APP_NAME="auth-service"

deps:
	@cargo install cargo-watch

dev:
	cargo watch -x 'run --bin $(APP_NAME)'
