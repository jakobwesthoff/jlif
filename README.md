# jlif - JSON Line Formatter

A command-line tool for quickly scanning multi-object JSON logs and data streams, handling both single-line and multi-line JSON objects mixed with non-JSON content.

## Overview

jlif processes continuous text streams from standard input, intelligently detecting JSON objects that may span multiple lines (such as pretty-printed JSON in logs), and formats them for improved readability while preserving non-JSON content unchanged. This makes it ideal for scanning application logs, API responses, and data streams where JSON objects are mixed with other text content.

## Installation

Pre-built binaries for macOS, Linux, and Windows are available in the [GitHub Releases](https://github.com/jakobwesthoff/jlif/releases) section.

## Usage

Basic usage:
```bash
# Pretty-print JSON with colors
tail -f app.log | jlif

# HTTP response with headers and JSON
curl -si https://randomuser.me/api/ | jlif

# Compact output without colors
cat data.jsonl | jlif -c --no-color

# Filter for error messages
kubectl logs pod | jlif -f "error"

# Show only JSON, hide other log lines
docker logs container | jlif -j
```

Use `jlif --help` for all available options.

## Building

```bash
cargo build --release
```
