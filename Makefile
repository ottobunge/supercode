.PHONY: build run test clean setup lint check

# Default target
all: build

# Build the project
build:
	cargo build --release

# Build in debug mode
build-debug:
	cargo build

# Run the MCP server
run:
	cargo run -- serve --port 9091

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Generate opencode.json config
setup:
	./scripts/generate-opencode-config.sh

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Run cargo check
check:
	cargo check

# Format code
fmt:
	cargo fmt

# List sessions
sessions:
	cargo run -- sessions
