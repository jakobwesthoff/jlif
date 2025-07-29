use crate::buffer::{BufferResult, LineBuffer};
use anyhow::Result;
use std::io::{BufRead, BufReader, Read, Write};

pub struct StreamProcessor<R: Read, W: Write> {
    reader: BufReader<R>,
    writer: W,
    buffer: LineBuffer,
}

impl<R: Read, W: Write> StreamProcessor<R, W> {
    pub fn new(reader: R, writer: W, buffer: LineBuffer) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
            buffer,
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
            match result {
                BufferResult::Json(json_value) => {
                    // Output JSON as compact single-line string
                    writeln!(self.writer, "{}", serde_json::to_string(&json_value)?)?;
                }
                BufferResult::Text(text) => {
                    // Output text as-is
                    writeln!(self.writer, "{}", text)?;
                }
                BufferResult::Incomplete(_) => {
                    // Ignore incomplete results - they're still being processed
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut processor = StreamProcessor::new(Cursor::new(input), &mut output, buffer);

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
        let mut processor = StreamProcessor::new(Cursor::new(input), &mut output, buffer);

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
        let mut processor = StreamProcessor::new(Cursor::new(input), &mut output, buffer);

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
        let mut processor = StreamProcessor::new(Cursor::new(input), &mut output, buffer);

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
        let mut processor = StreamProcessor::new(Cursor::new(input), &mut output, buffer);

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
}
