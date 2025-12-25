SHELL := /bin/sh

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-10s %s\n", $$1, $$2}'

lint: ## Run clippy lints
	@cargo clippy --all-targets --all-features -- -D warnings

build: ## Build the project
	@cargo build

test: ## Run tests
	@cargo test

run: ## Run the application
	@cargo run

validate: ## Run integration API workflow test (starts server internally)
	@./scripts/validate_with_server.sh

all: build test lint validate ## Build, test, lint, and validate

.PHONY: run build test validate lint all help
