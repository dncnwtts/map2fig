.PHONY: build test test-fuzz clean help

help:
	@echo "HEALPix Plotter - Available targets:"
	@echo "  make build          - Build the project (cargo build)"
	@echo "  make test           - Run tests and clean artifacts"
	@echo "  make test-fuzz      - Run fuzzing tests (requires cargo +nightly)"
	@echo "  make clean          - Remove build artifacts and test outputs"
	@echo "  make help           - Show this message"

build:
	cargo build --release

test:
	./tools/scripts/run_tests.sh

test-fuzz:
	cargo +nightly fuzz run fuzz_projection
	cargo +nightly fuzz run fuzz_scaling
	cargo +nightly fuzz run fuzz_coordinate_transform

clean:
	cargo clean
	rm -f examples/output/*.png examples/output/*.pdf examples/output/*.txt
	@echo "Cleaned build artifacts and test outputs"
