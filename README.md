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

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `--max-lines <N>` | Max lines to buffer for multi-line JSON | 10 |
| `-f, --filter <PATTERN>` | Regex filter pattern | — |
| `-s, --case-sensitive` | Case-sensitive filtering | Off |
| `-v, --invert-match` | Invert filter (show non-matching) | Off |
| `-j, --json-only` | Show only JSON content | Off |
| `-c, --compact` | Compact single-line output | Off |
| `--no-color` | Disable syntax highlighting | Off |

## Filtering

Filter flags can be combined:

| Flags | Result |
|-------|--------|
| `-f "error"` | Show lines containing "error" |
| `-f "error" -j` | Show only JSON containing "error" |
| `-f "error" -v` | Show lines NOT containing "error" |

The filter pattern matches against the serialized JSON string, not individual fields. Filtering is case-insensitive by default; use `-s` for case-sensitive matching.

## Multi-line JSON

jlif automatically detects and assembles multi-line JSON objects. When a line starts with `{` or `[` but isn't valid JSON, jlif buffers lines until a complete object forms or the buffer limit is reached.

Use `--max-lines` to increase the buffer for deeply nested JSON.

## Examples

**Kubernetes logs:**
```bash
kubectl logs -f my-pod | jlif -f "error" -j
```

**Docker logs:**
```bash
docker logs -f container 2>&1 | jlif -j
```

**Pipe to jq:**
```bash
cat logs.jsonl | jlif -c --no-color | jq '.message'
```

## Building

```bash
cargo build --release
```

## License

This project is licensed under the Mozilla Public License 2.0 - see the [LICENSE](LICENSE) file for details.
