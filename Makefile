.PHONY: all build release run clean install help

all: build

build:
	cargo build

release:
	cargo build --release

run:
	cargo run --release

clean:
	cargo clean

install: release
	install -m 755 target/release/emon-tui /usr/local/bin/emon-tui

help:
	@echo "emon-tui Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build    - Build the project (debug)"
	@echo "  make release  - Build the project (optimized)"
	@echo "  make run      - Build and run the release binary"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make install  - Install to /usr/local/bin (requires sudo)"
	@echo "  make help     - Show this help message"
