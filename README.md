# jlif - JSON Line Formatter

A command-line tool for processing and formatting JSON data from streaming input sources.

## Overview

jlif processes continuous text streams from standard input, identifies JSON objects (both single-line and multi-line), and formats them for improved readability while passing through non-JSON content unchanged.

## Installation

TODO: Installation instructions

## Usage

TODO: Usage examples

## Planned Features

- Stream processing from stdin with real-time output
- Multi-line JSON object detection and parsing
- Pretty-printed JSON output with proper indentation
- Configurable line buffering for multi-line JSON (`--max-lines`)
- Pass-through for non-JSON content
- Regex-based filtering (`--filter`)
- Colorized JSON output
- Filter non json on users request

## Building

```bash
cargo build --release
```
