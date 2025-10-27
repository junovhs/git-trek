.PHONY: help build install run test clean release

help: ## Show this help message
	@echo "🚀 git-trek - Available commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build

install: ## Install git-trek locally
	./install.sh

run: ## Run git-trek
	cargo run

test: ## Run tests
	cargo test

clean: ## Clean build artifacts
	cargo clean

release: ## Build optimized release binary
	cargo build --release
	@echo "✅ Release binary at: target/release/git-trek"