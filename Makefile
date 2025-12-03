.PHONY: help build test clean check clippy fmt doc release install run watch audit coverage all setup demo-network init-chain init-node start-bootnode start-node docker-build docker-run docker-stop

# Default target
help: ## Show this help message
	@echo "VCBC Blockchain - Development Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Project-specific targets:"
	@echo "  demo-network   Setup a complete demo network (bootnode + 2 nodes)"
	@echo "  init-chain     Initialize a new blockchain network"
	@echo "  init-node      Initialize a new node"
	@echo "  start-bootnode Start a bootnode"
	@echo "  start-node     Start a regular node"

# Development targets
build: ## Build the project in debug mode
	cargo build

test: ## Run all tests
	cargo test

test-unit: ## Run unit tests only (no integration tests)
	cargo test --lib

test-integration: ## Run integration tests only
	cargo test --test '*'

test-specific: ## Run a specific test (usage: make test-specific TEST=test_name)
	cargo test $(TEST)

clean: ## Clean build artifacts
	cargo clean

check: ## Check the code for compilation errors
	cargo check

clippy: ## Run clippy linter with strict warnings
	cargo clippy -- -D warnings

fmt: ## Format the code using rustfmt
	cargo fmt

fmt-check: ## Check code formatting without modifying files
	cargo fmt --check

doc: ## Generate documentation
	cargo doc --open --no-deps

doc-private: ## Generate documentation including private items
	cargo doc --document-private-items --open --no-deps

# Release targets
release: ## Build the project in release mode
	cargo build --release

install: ## Install the binary to ~/.cargo/bin
	cargo install --path .

# Development helpers
run: ## Run the project
	cargo run

run-release: ## Run the project in release mode
	cargo run --release

watch: ## Run the project with file watching (requires cargo-watch)
	cargo watch -x run

watch-test: ## Run tests with file watching (requires cargo-watch)
	cargo watch -x test

# Quality assurance
audit: ## Audit dependencies for security vulnerabilities
	cargo audit

all: check test clippy fmt ## Run all quality checks (check, test, clippy, fmt)

# Coverage (requires cargo-tarpaulin)
coverage: ## Generate code coverage report
	cargo tarpaulin --out Html --output-dir coverage
	@echo "Coverage report generated in coverage/tarpaulin-report.html"

coverage-report: ## Generate coverage report in terminal
	cargo tarpaulin --out Stdout

# Setup and development environment
setup: ## Setup development environment
	rustup component add clippy rustfmt
	cargo install cargo-audit cargo-tarpaulin cargo-watch git-cliff
	cargo install cargo-release
	@echo "Development environment setup complete"

# VCBC-specific targets for blockchain operations
init-chain: ## Initialize a new blockchain network
	@echo "Initializing new blockchain network..."
	cargo run -- init-chain --network-id vcbc-demo --chain-id 1

init-authority: ## Initialize network authority for bootnode certificates
	@echo "Initializing network authority..."
	cargo run -- init-authority --name vcbc-demo-authority --output authority.json

register-bootnode: ## Register and certify a bootnode
	@echo "Registering bootnode..."
	cargo run -- register-bootnode --node-id demo-bootnode --authority authority.json --config bootnode.json

init-node: ## Initialize a new node for existing network
	@echo "Initializing new node..."
	cargo run -- init-node --bootstrap-url http://localhost:8080 --config node.json

start-bootnode: ## Start a certified bootnode
	@echo "Starting bootnode..."
	cargo run -- start-bootnode --config bootnode.json

start-node: ## Start a regular node
	@echo "Starting regular node..."
	cargo run -- start-node --config node.json

# Demo network setup
demo-network: init-authority register-bootnode ## Setup a complete demo network
	@echo ""
	@echo "Demo network setup complete!"
	@echo ""
	@echo "To start the network:"
	@echo "1. Terminal 1: make start-bootnode"
	@echo "2. Terminal 2: make init-node && make start-node"
	@echo "3. Terminal 3: make init-node && make start-node CONFIG=node2.json"
	@echo ""
	@echo "Bootnode config: bootnode.json"
	@echo "Authority config: authority.json"
	@echo "Node configs: node.json, node2.json"

# Docker targets
docker-build: ## Build Docker image
	docker build -t vcbc:latest .

docker-run: ## Run the application in Docker
	docker run -p 8080:8080 -p 9090:9090 vcbc:latest

docker-stop: ## Stop all running vcbc containers
	docker stop $$(docker ps -q --filter ancestor=vcbc) || true

docker-clean: ## Remove vcbc Docker images and containers
	docker rm $$(docker ps -a -q --filter ancestor=vcbc) || true
	docker rmi vcbc:latest || true

# Release targets (requires cargo-release)
release-patch: ## Release a patch version
	cargo release patch --execute

release-minor: ## Release a minor version
	cargo release minor --execute

release-major: ## Release a major version
	cargo release major --execute

release-dry-run: ## Dry run release to see what would happen
	cargo release --dry-run

# Changelog generation (requires git-cliff)
changelog: ## Generate changelog
	git-cliff --latest --strip all > CHANGELOG.md

changelog-unreleased: ## Generate changelog with unreleased section
	git-cliff > CHANGELOG.md

# Utility targets
update-deps: ## Update all dependencies
	cargo update

outdated: ## Check for outdated dependencies
	cargo outdated

analyze-commits: ## Analyze recent commits for release determination
	./scripts/analyze-commits.sh development

tree: ## Show project structure
	tree -I target

lines: ## Count lines of code
	find src -name "*.rs" -exec wc -l {} + | tail -1

# Benchmarking (requires cargo-criterion)
bench: ## Run benchmarks
	cargo bench

bench-compare: ## Compare benchmark results
	cargo bench -- --save-baseline current
	cargo bench -- --baseline current

# CI/CD simulation
ci: all audit ## Run full CI pipeline
	@echo "CI pipeline completed successfully!"

# Development workflow
dev: fmt clippy test ## Development workflow: format, lint, test

# Quick development cycle
quick: check test ## Quick check: compile and test
