.PHONY: help test build release run clean fmt clippy check docs all

help:
	@echo "Lisp Interpreter - Makefile targets:"
	@echo ""
	@echo "  make build      - Build debug version"
	@echo "  make release    - Build optimized release version"
	@echo "  make test       - Run all tests"
	@echo "  make run        - Run the REPL with full I/O enabled (files + network)"
	@echo "  make check      - Quick compile check"
	@echo "  make fmt        - Format code with rustfmt"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make docs       - Generate documentation"
	@echo "  make all        - Build + test + clippy + fmt"
	@echo ""

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --all

run:
	cargo run --release -- --fs-sandbox . --fs-sandbox ./data --fs-sandbox ./examples --fs-sandbox ./scripts --allow-network

check:
	cargo check

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features

clean:
	cargo clean

docs:
	cargo doc --no-deps --open

all: clean fmt clippy test build release
	@echo "✅ All checks passed!"
