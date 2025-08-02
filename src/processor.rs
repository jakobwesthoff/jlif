use crate::buffer::{BufferResult, LineBuffer};
use crate::filter::{Filter, FilterInput, OutputFilter};
use crate::formatter::{Formatter, JsonFormatter};
use anyhow::Result;
use std::io::{BufRead, BufReader, Read, Write};

pub struct StreamProcessor<R: Read, W: Write> {
    reader: BufReader<R>,
    writer: W,
    buffer: LineBuffer,
    filter: OutputFilter,
    json_formatter: JsonFormatter,
}

impl<R: Read, W: Write> StreamProcessor<R, W> {
    pub fn new(
        reader: R,
        writer: W,
        buffer: LineBuffer,
        filter: OutputFilter,
        json_formatter: JsonFormatter,
    ) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
            buffer,
            filter,
            json_formatter,
        }
    }

    /// Process the stream line by line until EOF, then drain remaining buffer
    pub fn process(&mut self) -> Result<()> {
        let mut line = String::new();

        // Read lines until EOF
        while self.reader.read_line(&mut line)? > 0 {
            // Remove trailing newline if present
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            // Process line through buffer
            let results = self.buffer.add_line(line.clone());
            self.handle_results(results)?;

            line.clear();
        }

        // Drain remaining buffered content at EOF
        let drain_results = self.buffer.drain();
        self.handle_results(drain_results)?;

        Ok(())
    }

    fn handle_results(&mut self, results: Vec<BufferResult>) -> Result<()> {
        for result in results {
            // Try to convert BufferResult to FilterInput
            // Incomplete results are automatically filtered out by the conversion
            if let Ok(filter_input) = FilterInput::try_from(&result) {
                // Apply filter to determine if content should be output
                if self.filter.matches(&filter_input) {
                    match result {
                        BufferResult::Json(json_value) => {
                            // Output JSON using the configured formatter
                            let json_string = self.json_formatter.format_json(&json_value)?;
                            writeln!(self.writer, "{}", json_string)?;
                        }
                        BufferResult::Text(text) => {
                            // Output text as-is
                            writeln!(self.writer, "{}", text)?;
                        }
                        BufferResult::Incomplete(_) => {
                            // This should never happen due to FilterInput::try_from filtering,
                            // but we handle it defensively
                        }
                    }
                }
                // If filter doesn't match, content is suppressed (no output)
            }
            // If conversion fails (Incomplete), content is not output
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{NoFilter, OutputFilter};
    use std::io::Cursor;

    #[test]
    fn test_process_mixed_content() {
        let input = r#"Regular text line
{"valid": "json", "number": 42}
Another text line
{
  "multiline": "json"
}
Final text line"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::None(NoFilter);
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        assert_eq!(
            lines,
            vec![
                "Regular text line",
                r#"{"valid":"json","number":42}"#, // Original key order preserved
                "Another text line",
                r#"{"multiline":"json"}"#,
                "Final text line"
            ]
        );
    }

    #[test]
    fn test_process_incomplete_json_at_eof() {
        let input = r#"Complete line
{
  "incomplete": "json without closing"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::None(NoFilter);
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        assert_eq!(
            lines,
            vec![
                "Complete line",
                "{",
                r#"  "incomplete": "json without closing"#
            ]
        );
    }

    #[test]
    fn test_process_only_json() {
        let input = r#"{"first": 1}
{"second": 2}
[1, 2, 3]
"string value""#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::None(NoFilter);
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        assert_eq!(
            lines,
            vec![
                r#"{"first":1}"#,
                r#"{"second":2}"#,
                r#"[1,2,3]"#,
                r#""string value""#
            ]
        );
    }

    #[test]
    fn test_process_empty_input() {
        let input = "";

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::None(NoFilter);
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "");
    }

    #[test]
    fn test_process_with_overflow() {
        let input = r#"{invalid json start
more invalid
yet more invalid
{"valid": "json"}
final text"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(3); // Small buffer to trigger overflow
        let filter = OutputFilter::None(NoFilter);
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        assert_eq!(
            lines,
            vec![
                "{invalid json start",
                "more invalid",
                "yet more invalid",
                r#"{"valid":"json"}"#,
                "final text"
            ]
        );
    }

    #[test]
    fn test_process_with_regex_filter() {
        let input = r#"Regular text line
{"status": "error", "message": "failed"}
Info: everything ok
{"status": "ok", "message": "success"}
ERROR: critical system failure"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::from_args(Some("error".to_string()), false, false).unwrap();
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        // Should only output lines containing "error" (case-insensitive)
        assert_eq!(
            lines,
            vec![
                r#"{"status":"error","message":"failed"}"#,
                "ERROR: critical system failure"
            ]
        );
    }

    #[test]
    fn test_process_with_case_sensitive_filter() {
        let input = r#"error: lowercase
ERROR: uppercase
Error: mixed case
info: no match"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::from_args(Some("ERROR".to_string()), true, false).unwrap();
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        // Should only output lines with exact case match
        assert_eq!(lines, vec!["ERROR: uppercase"]);
    }

    #[test]
    fn test_process_filter_json_structure() {
        let input = r#"{"status": "error", "code": 500}
{"status": "ok", "code": 200}
{"error": "not matching"}
Plain text with status error"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        // Filter for JSON objects with status: error pattern
        let filter =
            OutputFilter::from_args(Some(r#""status"\s*:\s*"error""#.to_string()), false, false)
                .unwrap();
        let formatter = JsonFormatter::from_args(true, true); // compact, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = output_str.trim().split('\n').collect();

        // Should only match the first JSON object
        assert_eq!(lines, vec![r#"{"status":"error","code":500}"#]);
    }

    #[test]
    fn test_process_pretty_printed_output() {
        let input = r#"{"name": "Deep Space Nine", "location": "Bajoran system"}
Text line
{"crew": {"captain": "Sisko", "science": "Dax"}}"#;

        let mut output = Vec::new();
        let buffer = LineBuffer::new(10);
        let filter = OutputFilter::None(NoFilter);
        let formatter = JsonFormatter::from_args(false, true); // pretty, no_color
        let mut processor =
            StreamProcessor::new(Cursor::new(input), &mut output, buffer, filter, formatter);

        processor.process().unwrap();

        let output_str = String::from_utf8(output).unwrap();

        // Should contain pretty-printed JSON with indentation
        assert!(output_str.contains("{\n  \"name\": \"Deep Space Nine\""));
        assert!(output_str.contains("  \"location\": \"Bajoran system\"\n}"));
        assert!(output_str.contains("{\n  \"crew\": {\n    \"captain\": \"Sisko\""));
        assert!(output_str.contains("Text line"));
    }
}
