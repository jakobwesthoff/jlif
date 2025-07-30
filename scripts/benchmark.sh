#!/usr/bin/env bash

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Create temporary large test file
TEMP_FILE="$(mktemp)"

cleanup() {
    # Cleanup
    rm "$TEMP_FILE"
    echo "Cleanup completed"
}

trap "cleanup" EXIT

echo "Creating large test file: $TEMP_FILE"
# Concatenate test.json to a large length
# Use yes + head for fast repetition
yes "$(cat "${SCRIPT_DIR}/test.json")" | head -n 2000000 >"$TEMP_FILE" || true

INPUT_LINES=$(wc -l <"$TEMP_FILE")
echo "Generated test file with $INPUT_LINES lines"

# Build release version
echo "Building release version..."
cargo build --release

# Run benchmarks with different max-lines values
MAX_LINES_VALUES=(8 16 32 64 128)

echo "Running benchmarks with different max-lines values..."
for max_lines in "${MAX_LINES_VALUES[@]}"; do
    echo ""
    echo "=== Benchmarking with --max-lines $max_lines ==="

    hyperfine \
        --runs 5 \
        --warmup 1 \
        --prepare "echo 'Starting benchmark run with max-lines $max_lines...'" \
        --export-json "benchmark_results_${max_lines}.json" \
        "cat '$TEMP_FILE' | ./target/release/jlif --max-lines $max_lines"

    # Calculate and display lines per second for this run
    if command -v jq >/dev/null 2>&1; then
        MEAN_TIME=$(jq -r '.results[0].mean' "benchmark_results_${max_lines}.json")
        LINES_PER_SEC=$(echo "scale=0; $INPUT_LINES / $MEAN_TIME" | bc -l 2>/dev/null || echo "0")
        echo "Performance Summary (max-lines $max_lines):"
        echo "  Lines processed: $INPUT_LINES"
        echo "  Average time: ${MEAN_TIME}s"
        echo "  Lines per second: $LINES_PER_SEC"
    fi
done

echo ""
echo "=== All benchmarks completed ==="
