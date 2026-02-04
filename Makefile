SHELL := /bin/sh

ENV ?= local

ifeq ($(ENV),local)
MONGO_URI ?= mongodb://127.0.0.1:27017
MONGO_DB ?= verbumdei
QUESTIONS_COLLECTION ?= questions
ERAS_COLLECTION ?= eras
QUESTIONS_DATA_FILE ?= data/questions.json
ERAS_DATA_FILE ?= data/eras.json
QUESTIONS_OUT_FILE ?= data/questions.json
ERAS_OUT_FILE ?= data/eras.json
endif

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-10s %s\n", $$1, $$2}'

lint: ## Run clippy lints
	@cargo clippy --all-targets --all-features -- -D warnings

fmt: ## Format Rust code
	@cargo fmt

fmt-check: ## Check Rust formatting
	@cargo fmt -- --check

check: ## Run lint and format checks
	@$(MAKE) lint
	@$(MAKE) fmt-check

build: ## Build the project
	@cargo build

test: ## Run tests
	@cargo test

run-dev: ## Run the application in development mode
	@ENV=local RUST_LOG=debug cargo watch -x run

run: ## Run the application
	@cargo run

validate: ## Run integration API workflow test (starts server internally)
	@./scripts/validate_with_server.sh

load-questions: ## Load questions from data/questions.json into Mongo
	@MONGO_URI="$(MONGO_URI)" MONGO_DB="$(MONGO_DB)" COLLECTION="$(QUESTIONS_COLLECTION)" DATA_FILE="$(QUESTIONS_DATA_FILE)" ./scripts/load_questions.sh

load-eras: ## Load eras from data/eras.json into Mongo
	@MONGO_URI="$(MONGO_URI)" MONGO_DB="$(MONGO_DB)" COLLECTION="$(ERAS_COLLECTION)" DATA_FILE="$(ERAS_DATA_FILE)" ./scripts/load_eras.sh

load-data: load-questions load-eras ## Load all collection data into Mongo

dump-questions: ## Dump questions from Mongo into data/questions.json
	@MONGO_URI="$(MONGO_URI)" MONGO_DB="$(MONGO_DB)" COLLECTION="$(QUESTIONS_COLLECTION)" OUT_FILE="$(QUESTIONS_OUT_FILE)" ./scripts/dump_questions.sh

dump-eras: ## Dump eras from Mongo into data/eras.json
	@MONGO_URI="$(MONGO_URI)" MONGO_DB="$(MONGO_DB)" COLLECTION="$(ERAS_COLLECTION)" OUT_FILE="$(ERAS_OUT_FILE)" ./scripts/dump_eras.sh

dump-data: dump-questions dump-eras ## Dump all collection data from Mongo

all: build test lint fmt-check validate ## Build, test, lint, format-check, and validate

.PHONY: run build test validate lint fmt fmt-check check \
	load-questions load-eras load-data \
	dump-questions dump-eras dump-data \
	all help
