SHELL := /bin/sh

ENV ?= local

ifeq ($(ENV),local)
MONGO_URI ?= mongodb://127.0.0.1:27017
MONGO_DB ?= verbumdei
COLLECTION ?= questions
DATA_FILE ?= data/questions.json
OUT_FILE ?= data/questions.json
endif

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

load-data: ## Load questions from data/questions.json into Mongo
	@MONGO_URI="$(MONGO_URI)" MONGO_DB="$(MONGO_DB)" COLLECTION="$(COLLECTION)" DATA_FILE="$(DATA_FILE)" ./scripts/load_questions.sh

dump-data: ## Dump questions from Mongo into data/questions.json
	@MONGO_URI="$(MONGO_URI)" MONGO_DB="$(MONGO_DB)" COLLECTION="$(COLLECTION)" OUT_FILE="$(OUT_FILE)" ./scripts/dump_questions.sh

all: build test lint validate ## Build, test, lint, and validate

.PHONY: run build test validate lint load-data dump-data all help
