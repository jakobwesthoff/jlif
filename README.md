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

<!-- docs:start -->
## Documentation

jlif reads from stdin and writes to stdout. Pipe any input containing JSON through it to get formatted, syntax-highlighted output. Non-JSON lines pass through unchanged by default.

```bash
command | jlif [OPTIONS]
```

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `--max-lines <N>` | Max lines to buffer for multi-line JSON | 10 |
| `-f, --filter <PATTERN>` | Regex filter pattern | — |
| `-s, --case-sensitive` | Case-sensitive filtering | Off |
| `-v, --invert-match` | Invert filter (show non-matching) | Off |
| `-j, --json-only` | Show only JSON content | Off |
| `-c, --compact` | Compact single-line output | Off |
| `--no-color` | Disable syntax highlighting | Off |
| `-h, --help` | Print help | — |
| `-V, --version` | Print version | — |

### Examples

```bash
# Pretty-print JSON from stdin
echo '{"name":"test","value":42}' | jlif

# Filter for specific patterns (case-insensitive by default)
cat logs.jsonl | jlif -f "error|warning"

# Case-sensitive filtering
cat logs.jsonl | jlif -f "Error" -s

# Show everything except matching lines
cat logs.jsonl | jlif -f "debug" -v

# Compact output without colors (for piping)
cat data.json | jlif -c --no-color | jq '.field'

# Only show JSON, skip non-JSON lines
tail -f mixed.log | jlif -j

# Handle multi-line JSON with larger buffer
cat pretty.json | jlif --max-lines 50
```

### Multi-line JSON Support

jlif automatically detects and assembles multi-line JSON objects. When a line starts with `{` or `[` but isn't valid JSON, jlif buffers subsequent lines until a complete JSON object is formed or the buffer limit is reached.

```json
{
  "event": "request",
  "data": {
    "method": "POST",
    "path": "/api/users"
  }
}
```

Use `--max-lines` to adjust the buffer size for deeply nested or heavily formatted JSON. The default of 10 lines handles most cases.

### Pass-through Behavior

Non-JSON content passes through unchanged by default, making jlif work well with mixed log formats:

```bash
$ cat mixed.log | jlif
2024-01-15 Starting server...
{
  "level": "info",
  "message": "Server listening on port 8080"
}
Connection established
{
  "level": "debug",
  "client": "192.168.1.1"
}
```

Use `-j` / `--json-only` to suppress non-JSON lines and show only formatted JSON objects.

### How Filtering Works

The filter flags (`-f`, `-j`, `-v`) can be combined, and they compose in a specific way:

| Flags | Result |
|-------|--------|
| `-f "error"` | Show lines (JSON or non-JSON) containing "error" |
| `-f "error" -j` | Show only JSON objects containing "error" |
| `-f "error" -v` | Show lines NOT containing "error" |
| `-j` | Show all JSON, hide non-JSON |

#### Regex Matching

The filter pattern matches against the **serialized JSON string**, not individual fields. This means:

```bash
# This JSON object:
{"level": "error", "message": "Connection failed"}

# Matches all of these patterns:
-f "error"           # Matches "error" in level field
-f "Connection"      # Matches in message field
-f "level.*error"    # Regex across the serialized string
-f '"level"'         # Matches the literal field name with quotes
```

Filtering is case-insensitive by default. Use `-s` for case-sensitive matching.

### Buffer Behavior

jlif uses a smart buffering system to handle multi-line JSON:

- When a line starts with `{`, `[`, or `"` but isn't valid JSON, jlif starts buffering
- Lines are accumulated until they form valid JSON or the buffer limit is reached
- If the buffer fills without forming valid JSON, the oldest lines are output as non-JSON and jlif tries to extract any valid JSON from the remaining content

Use `--max-lines` to increase the buffer for deeply nested JSON. The default of 10 lines handles most structured logs.

> **When to Increase --max-lines**
>
> Increase the buffer if you're processing pretty-printed JSON with many levels of nesting, or if you see JSON objects being split across multiple outputs.

### Real-World Examples

**Kubernetes pod logs:**
```bash
# Filter for errors in a specific pod
kubectl logs -f my-pod | jlif -f "error|exception" -j

# Show only non-error logs
kubectl logs my-pod | jlif -f "error" -v

# Follow logs from multiple pods, JSON only
kubectl logs -l app=myapp -f | jlif -j
```

**Docker container logs:**
```bash
# Filter docker-compose logs by service pattern
docker-compose logs -f | jlif -f "api|worker"

# Show only JSON from a specific container
docker logs -f my-container 2>&1 | jlif -j

# Exclude health check logs
docker logs my-container | jlif -f "health" -v
```

**API server logs:**
```bash
# Find requests with specific status codes
tail -f /var/log/api/access.log | jlif -f '"status":5'

# Track a correlation ID across services
cat *.log | jlif -f "correlation.*abc-123"

# Show only slow requests (assuming duration field)
tail -f app.log | jlif -f '"duration":[0-9]{4,}'
```

**Piping to other tools:**
```bash
# Extract specific field with jq
cat logs.jsonl | jlif -c --no-color | jq -r '.message'

# Count error types
cat logs.jsonl | jlif -f "error" -c --no-color | jq -r '.error_type' | sort | uniq -c

# Pretty-print then search with grep
cat logs.jsonl | jlif | grep -A5 "Connection failed"
```

### Error Handling

jlif handles malformed input gracefully:

- **Invalid JSON**: Passed through as non-JSON text (unless `-j` is used)
- **Incomplete JSON at EOF**: Buffered content is discarded if it doesn't form valid JSON
- **Invalid regex pattern**: jlif exits with an error message
- **Binary data**: May produce unexpected output; jlif expects UTF-8 text input

<!-- docs:end -->

## Building

```bash
cargo build --release
```

## License

This project is licensed under the Mozilla Public License 2.0 - see the [LICENSE](LICENSE) file for details.
