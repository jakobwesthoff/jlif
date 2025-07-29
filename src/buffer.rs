use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum BufferResult {
    Json(Value),             // Parsed JSON object ready for formatting
    Text(String),            // Non-JSON line for pass-through
    Incomplete(Vec<String>), // Buffered lines, need more input
}

pub struct LineBuffer {
    buffer: Vec<String>,
    max_lines: usize,
}

impl LineBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_lines,
        }
    }

    /// Processes a new line and returns parsing results.
    ///
    /// ## Processing Logic Overview
    ///
    /// We use a state machine to handle different parsing scenarios:
    ///
    /// ### Accumulating State
    /// - **Full Buffer Parsing**: Try to parse entire buffer as single JSON
    ///   - Example: buffer `["{", "\"key\": \"value\"", "}"]` → parses as complete JSON object
    ///   - If successful: extract JSON, clear buffer, continue in Accumulating state
    ///
    /// - **Overflow Handling**: When buffer exceeds max_lines (e.g., max_lines=3, buffer has 3+ lines)
    ///   - Example: buffer `["garbage", "{\"a\":1}", "more text", "fourth line"]` with max_lines=3
    ///   - Action: remove first line ("garbage") as Text, switch to Draining state
    ///   - Rationale: We can't wait forever, must make progress by removing oldest line
    ///
    /// - **Single Non-JSON Flush**: When buffer has exactly 1 line that can't be JSON
    ///   - Example: buffer `["regular text line"]` where "regular text line" doesn't start with {,[,"
    ///   - Action: flush as Text immediately since it can never become JSON
    ///
    /// ### Draining State  
    /// (Entered after removing a line - actively extracting content from buffer)
    ///
    /// - **Forward Scanning**: Try parsing from start, growing segments: [0..1], [0..2], [0..3]...
    ///   - Example: buffer `["{\"a\":1}", "text", "{", "}"]` after overflow
    ///   - Try `["{\"a\":1}"]` → valid JSON! Extract it, remove 1 line, STAY in Draining
    ///   - Buffer now `["text", "{", "}"]` - structure changed again, continue Draining processing
    ///   - Next iteration: "text" not JSON-like → flush as Text, STAY in Draining  
    ///   - Buffer now `["{", "}"]` - structure changed again, continue Draining processing
    ///   - Next iteration: try `["{", "}"]` → valid JSON! Extract it, buffer empty, done
    ///
    /// - **Non-JSON First Line**: If first line after overflow isn't JSON-like
    ///   - Example: buffer `["plain text", "{\"a\":1}"]` after overflow  
    ///   - Action: flush "plain text" as Text, STAY in Draining (buffer structure changed)
    ///
    /// - **No Progress**: First line could be JSON but forward scan finds nothing
    ///   - Example: buffer `["{incomplete", "json"]` - looks like JSON start but isn't complete
    ///   - Action: back to Accumulating state (buffer structure unchanged, wait for more input)
    ///
    /// ### Key Insight
    /// - Accumulating state: conservative, building up content until complete structures emerge
    /// - Draining state: aggressive, keeps extracting content until buffer structure stops changing
    /// - Draining only returns to Accumulating when no modifications are made to the buffer
    /// - This ensures we extract all possible JSON after any buffer structure change
    pub fn add_line(&mut self, line: String) -> Vec<BufferResult> {
        // Quick shortcut: if buffer is empty and line doesn't start with JSON chars
        if self.buffer.is_empty() && !Self::could_be_json_start(&line) {
            return vec![BufferResult::Text(line)];
        }

        self.buffer.push(line);
        let mut results = Vec::new();

        #[derive(Debug)]
        enum BufferState {
            Accumulating, // Building up content, being patient - try full buffer parsing
            Draining,     // Actively removing content after overflow - try forward scanning
        }

        let mut state = BufferState::Accumulating;

        // Keep processing until buffer is stable
        loop {
            let mut is_stable = true;

            match state {
                BufferState::Accumulating => {
                    if let Some((json_value, _)) = self.try_parse_buffer_segments() {
                        // Full buffer is JSON - no text before it
                        results.push(BufferResult::Json(json_value));
                        self.buffer.clear();
                        is_stable = false;
                    } else if self.buffer.len() >= self.max_lines {
                        // Overflow: remove first line and transition to Draining
                        results.push(BufferResult::Text(self.buffer.remove(0)));
                        state = BufferState::Draining;
                        is_stable = false;
                    } else if self.buffer.len() == 1 && !Self::could_be_json_start(&self.buffer[0])
                    {
                        // Single non-JSON line - flush it
                        results.push(BufferResult::Text(self.buffer.remove(0)));
                        is_stable = false;
                    }
                }
                BufferState::Draining => {
                    if let Some((json_value, end_idx)) = self.try_parse_forward_segments() {
                        // Found JSON via forward scanning
                        results.push(BufferResult::Json(json_value));
                        for _ in 0..end_idx {
                            self.buffer.remove(0);
                        }
                        state = BufferState::Draining; // Stay in draining - buffer structure changed
                        is_stable = false;
                    } else if !Self::could_be_json_start(&self.buffer[0]) {
                        // First line not JSON-like, flush as text
                        results.push(BufferResult::Text(self.buffer.remove(0)));
                        state = BufferState::Draining; // Stay in draining - buffer structure changed
                        is_stable = false;
                    } else {
                        // First line could be JSON but forward scan found nothing
                        // Buffer structure unchanged - back to accumulating
                        state = BufferState::Accumulating;
                    }
                }
            }

            // If buffer is stable, we're done
            if is_stable {
                if !self.buffer.is_empty() {
                    results.push(BufferResult::Incomplete(self.buffer.clone()));
                }
                break;
            }

            // If buffer is empty, we're done
            if self.buffer.is_empty() {
                break;
            }
        }

        results
    }

    fn could_be_json_start(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return false;
        }

        match trimmed {
            s if s.starts_with('"') || s.starts_with('{') || s.starts_with('[') => true,
            s if s.starts_with("true") || s.starts_with("false") || s.starts_with("null") => true,
            s if s.starts_with(|c: char| c.is_ascii_digit() || c == '-') => true,
            _ => false,
        }
    }

    fn try_parse_buffer_segments(&self) -> Option<(Value, usize)> {
        // Only try full buffer parsing
        let full_combined = self.buffer.join("\n");
        if let Ok(json_value) = serde_json::from_str::<Value>(&full_combined) {
            return Some((json_value, 0));
        }

        None
    }

    fn try_parse_forward_segments(&self) -> Option<(Value, usize)> {
        // Forward scan from start: [0..1], [0..2], [0..3], etc.
        for end_idx in 1..=self.buffer.len() {
            let segment = &self.buffer[0..end_idx];
            let combined = segment.join("\n");

            if let Ok(json_value) = serde_json::from_str::<Value>(&combined) {
                return Some((json_value, end_idx));
            }
        }

        None
    }

    /// Drains all remaining buffer contents, extracting any valid JSON.
    ///
    /// This method should be called when input ends (EOF) to flush any remaining
    /// buffered content. It follows the same logic as overflow draining but is
    /// more aggressive - it doesn't wait for potential JSON completion and
    /// flushes everything that can't be parsed as text.
    pub fn drain(&mut self) -> Vec<BufferResult> {
        let mut results = Vec::new();

        // Keep processing until buffer is empty (like Draining state)
        while !self.buffer.is_empty() {
            if let Some((json_value, end_idx)) = self.try_parse_forward_segments() {
                // Found valid JSON, extract it
                results.push(BufferResult::Json(json_value));
                for _ in 0..end_idx {
                    self.buffer.remove(0);
                }
            } else {
                // No valid JSON found, flush first line as text (don't wait)
                results.push(BufferResult::Text(self.buffer.remove(0)));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[test]
    fn test_quick_shortcut_non_json() {
        let mut buffer = LineBuffer::new(10);
        let results = buffer.add_line("Gul Dukat's therapy session, stardate 51721.3".to_string());

        assert_eq!(
            results,
            vec![BufferResult::Text(
                "Gul Dukat's therapy session, stardate 51721.3".to_string()
            )]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_single_line_json() {
        let mut buffer = LineBuffer::new(10);
        let results = buffer.add_line(
            r#"{"vedek": "Bareil Antos", "occupation": "Former resistance fighter"}"#.to_string(),
        );

        assert_eq!(
            results,
            vec![BufferResult::Json(
                json!({"vedek": "Bareil Antos", "occupation": "Former resistance fighter"})
            )]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_multi_line_json() {
        let mut buffer = LineBuffer::new(10);

        let results1 = buffer.add_line("{".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec!["{".to_string()])]
        );

        let results2 = buffer.add_line(r#"  "holosuite_program": "Vic Fontaine""#.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                "{".to_string(),
                r#"  "holosuite_program": "Vic Fontaine""#.to_string()
            ])]
        );

        let results3 = buffer.add_line("}".to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Json(
                json!({"holosuite_program": "Vic Fontaine"})
            )]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_json_like_text_before_json_non_greedy() {
        let mut buffer = LineBuffer::new(5);

        // Add JSON-like text that gets buffered due to starting with valid char
        let results1 = buffer.add_line("{Morn speaks for the first time}".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![
                "{Morn speaks for the first time}".to_string()
            ])]
        );

        // Add actual JSON - non-greedy mode waits to see if it forms complete structure
        let results2 =
            buffer.add_line(r#"{"patron": "Morn", "beverage_tab": "astronomical"}"#.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                "{Morn speaks for the first time}".to_string(),
                r#"{"patron": "Morn", "beverage_tab": "astronomical"}"#.to_string()
            ])]
        );

        // Add non-JSON line - still buffering in non-greedy mode
        let results3 = buffer.add_line("Quark closes the bar".to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Incomplete(vec![
                "{Morn speaks for the first time}".to_string(),
                r#"{"patron": "Morn", "beverage_tab": "astronomical"}"#.to_string(),
                "Quark closes the bar".to_string()
            ])]
        );

        // Add first line of multi-line JSON
        let results4 = buffer.add_line("{".to_string());
        assert_eq!(
            results4,
            vec![BufferResult::Incomplete(vec![
                "{Morn speaks for the first time}".to_string(),
                r#"{"patron": "Morn", "beverage_tab": "astronomical"}"#.to_string(),
                "Quark closes the bar".to_string(),
                "{".to_string()
            ])]
        );

        // Add second line with content and closing brace - this triggers overflow and completes the multi-line JSON
        let results5 = buffer.add_line(r#"  "barkeeper": "Quark"}"#.to_string());
        assert_eq!(
            results5,
            vec![
                BufferResult::Text("{Morn speaks for the first time}".to_string()),
                BufferResult::Json(json!({"patron": "Morn", "beverage_tab": "astronomical"})),
                BufferResult::Text("Quark closes the bar".to_string()),
                BufferResult::Json(json!({"barkeeper": "Quark"}))
            ]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_real_text_before_json() {
        let mut buffer = LineBuffer::new(10);

        // Add actual non-JSON text (quick shortcut applies)
        let results1 = buffer.add_line("Odo investigates a crime".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Text("Odo investigates a crime".to_string())]
        );

        // Add JSON - should parse normally
        let results2 = buffer.add_line(r#"{"suspect": "Quark", "evidence": "none"}"#.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Json(
                json!({"suspect": "Quark", "evidence": "none"})
            )]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_buffer_overflow() {
        let mut buffer = LineBuffer::new(2);

        let results1 = buffer.add_line("{".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec!["{".to_string()])]
        );

        let results2 = buffer.add_line("Weyoun-6 contemplating betraying the Dominion".to_string());
        assert_eq!(
            results2,
            vec![
                BufferResult::Text("{".to_string()),
                BufferResult::Text("Weyoun-6 contemplating betraying the Dominion".to_string())
            ]
        );

        let results3 = buffer.add_line("Damar drinking kanar alone".to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Text("Damar drinking kanar alone".to_string())]
        );
    }

    #[test]
    fn test_json_start_detection() {
        assert!(LineBuffer::could_be_json_start(
            r#"{"pagh": "wraith_of_Kahless"}"#
        ));
        assert!(LineBuffer::could_be_json_start(
            r#"["Ezri", "Jadzia", "Curzon", "Audrid"]"#
        ));
        assert!(LineBuffer::could_be_json_start(
            r#""Rule of Acquisition #34: War is good for business""#
        ));
        assert!(LineBuffer::could_be_json_start("true"));
        assert!(LineBuffer::could_be_json_start("false"));
        assert!(LineBuffer::could_be_json_start("null"));
        assert!(LineBuffer::could_be_json_start("47"));
        assert!(LineBuffer::could_be_json_start("-2375"));

        assert!(!LineBuffer::could_be_json_start("It is a good day to die"));
        assert!(!LineBuffer::could_be_json_start(""));
        assert!(!LineBuffer::could_be_json_start("   "));
    }

    #[rstest]
    #[case(r#"["Sisko", "Kira", "Dax"]"#, json!(["Sisko", "Kira", "Dax"]))]
    #[case(r#""The Prophets guide us""#, json!("The Prophets guide us"))]
    #[case("47", json!(47))]
    #[case("-2375", json!(-2375))]
    #[case("3.14159", json!(3.14159))]
    #[case("true", json!(true))]
    #[case("false", json!(false))]
    #[case("null", json!(null))]
    fn test_single_line_json_types(#[case] json_str: &str, #[case] expected: serde_json::Value) {
        let mut buffer = LineBuffer::new(10);
        let results = buffer.add_line(json_str.to_string());

        assert_eq!(results, vec![BufferResult::Json(expected)]);
        assert!(buffer.buffer.is_empty());
    }

    #[rstest]
    #[case("{invalid json syntax")]
    #[case("[incomplete array")]
    #[case(r#""unterminated string"#)]
    #[case("{Garak's mysterious past}")]
    #[case("[Odo's investigation, incomplete")]
    fn test_overflow_with_json_like_starts(#[case] json_like: &str) {
        let mut buffer = LineBuffer::new(2);

        // First add a line to fill buffer
        let results1 = buffer.add_line("{".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec!["{".to_string()])]
        );

        // Add JSON-like line that causes overflow
        // Correctly stays Incomplete because it could be completed by future lines
        let results2 = buffer.add_line(json_like.to_string());
        assert_eq!(
            results2,
            vec![
                BufferResult::Text("{".to_string()),
                BufferResult::Incomplete(vec![json_like.to_string()])
            ]
        );

        // Add non-JSON line - now JSON-like line gets flushed as text
        let results3 = buffer.add_line("Rom fixes the replicator".to_string());
        assert_eq!(
            results3,
            vec![
                BufferResult::Text(json_like.to_string()),
                BufferResult::Text("Rom fixes the replicator".to_string())
            ]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[rstest]
    #[case("[", r#"  "Worf","#, r#"  "Data""#, "]", json!(["Worf", "Data"]))]
    #[case("{", r#"  "species": "Klingon","#, r#"  "rank": "Lieutenant Commander""#, "}", json!({"species": "Klingon", "rank": "Lieutenant Commander"}))]
    fn test_multi_line_complete_json_structures(
        #[case] line1: &str,
        #[case] line2: &str,
        #[case] line3: &str,
        #[case] line4: &str,
        #[case] expected: serde_json::Value,
    ) {
        let mut buffer = LineBuffer::new(10);

        let results1 = buffer.add_line(line1.to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![line1.to_string()])]
        );

        let results2 = buffer.add_line(line2.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                line1.to_string(),
                line2.to_string()
            ])]
        );

        let results3 = buffer.add_line(line3.to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Incomplete(vec![
                line1.to_string(),
                line2.to_string(),
                line3.to_string()
            ])]
        );

        let results4 = buffer.add_line(line4.to_string());
        assert_eq!(results4, vec![BufferResult::Json(expected)]);
        assert!(buffer.buffer.is_empty());
    }

    #[rstest]
    #[case("[", r#""Bashir""#, "]", json!(["Bashir"]))]
    #[case("{", r#""doctor": "Julian Bashir""#, "}", json!({"doctor": "Julian Bashir"}))]
    #[case("[", "47", "]", json!([47]))]
    #[case("{", r#""number": 1701"#, "}", json!({"number": 1701}))]
    fn test_multi_line_with_valid_json_inside(
        #[case] open: &str,
        #[case] valid_json_content: &str,
        #[case] close: &str,
        #[case] expected: serde_json::Value,
    ) {
        let mut buffer = LineBuffer::new(10);

        let results1 = buffer.add_line(open.to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![open.to_string()])]
        );

        let results2 = buffer.add_line(valid_json_content.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                open.to_string(),
                valid_json_content.to_string()
            ])]
        );

        let results3 = buffer.add_line(close.to_string());
        assert_eq!(results3, vec![BufferResult::Json(expected)]);
        assert!(buffer.buffer.is_empty());
    }

    #[rstest]
    #[case("[", r#"{"name": "Quark"},"#, r#"{"name": "Rom"}"#, "]", json!([{"name": "Quark"}, {"name": "Rom"}]))]
    #[case("{", r#""crew": ["Sisko", "Kira"],"#, r#""station": "DS9""#, "}", json!({"crew": ["Sisko", "Kira"], "station": "DS9"}))]
    fn test_multi_line_with_complex_valid_json_inside(
        #[case] open: &str,
        #[case] valid_json1: &str,
        #[case] valid_json2: &str,
        #[case] close: &str,
        #[case] expected: serde_json::Value,
    ) {
        let mut buffer = LineBuffer::new(10);

        let results1 = buffer.add_line(open.to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![open.to_string()])]
        );

        let results2 = buffer.add_line(valid_json1.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                open.to_string(),
                valid_json1.to_string()
            ])]
        );

        let results3 = buffer.add_line(valid_json2.to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Incomplete(vec![
                open.to_string(),
                valid_json1.to_string(),
                valid_json2.to_string()
            ])]
        );

        let results4 = buffer.add_line(close.to_string());
        assert_eq!(results4, vec![BufferResult::Json(expected)]);
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_multiple_consecutive_overflows_mixed_json_types() {
        let mut buffer = LineBuffer::new(3);

        // Build up to first overflow
        let results1 = buffer.add_line("{Kai Winn plots against Sisko}".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![
                "{Kai Winn plots against Sisko}".to_string()
            ])]
        );

        let results2 = buffer.add_line(r#""Benjamin Sisko is the Emissary""#.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                "{Kai Winn plots against Sisko}".to_string(),
                r#""Benjamin Sisko is the Emissary""#.to_string()
            ])]
        );

        // First overflow occurs here - triggers draining cycle
        let results3 = buffer.add_line("[Prophets communicate through orbs".to_string());
        assert_eq!(
            results3,
            vec![
                BufferResult::Text("{Kai Winn plots against Sisko}".to_string()),
                BufferResult::Json(json!("Benjamin Sisko is the Emissary")),
                BufferResult::Incomplete(vec!["[Prophets communicate through orbs".to_string()])
            ]
        );

        // Add more content - still accumulating since no overflow
        let results4 = buffer.add_line(r#"[1, 2, 3]"#.to_string());
        assert_eq!(
            results4,
            vec![BufferResult::Incomplete(vec![
                "[Prophets communicate through orbs".to_string(),
                r#"[1, 2, 3]"#.to_string()
            ])]
        );

        // Second overflow occurs - triggers another draining cycle
        let results5 = buffer.add_line("Garak tailors clothes on the promenade".to_string());
        assert_eq!(
            results5,
            vec![
                BufferResult::Text("[Prophets communicate through orbs".to_string()),
                BufferResult::Json(json!([1, 2, 3])),
                BufferResult::Text("Garak tailors clothes on the promenade".to_string())
            ]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_empty_buffer_during_draining_state() {
        let mut buffer = LineBuffer::new(2);

        // Build up to overflow with one JSON-like line and one valid JSON
        let results1 = buffer.add_line("{invalid json syntax".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![
                "{invalid json syntax".to_string()
            ])]
        );

        // Adding second line triggers immediate overflow since max_lines=2
        // This processes the overflow and draining in the same call
        let results2 = buffer.add_line(r#"{"valid": "json"}"#.to_string());
        assert_eq!(
            results2,
            vec![
                BufferResult::Text("{invalid json syntax".to_string()),
                BufferResult::Json(json!({"valid": "json"}))
            ]
        );
        assert!(buffer.buffer.is_empty());

        // Verify that subsequent operations work correctly after buffer was emptied during draining
        let results3 = buffer.add_line("Normal text after draining".to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Text("Normal text after draining".to_string())]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_drain_mixed_content_with_valid_json() {
        let mut buffer = LineBuffer::new(10);

        // Add content that includes both invalid and valid JSON
        let results1 = buffer.add_line("{incomplete json".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![
                "{incomplete json".to_string()
            ])]
        );

        let results2 = buffer.add_line(r#"{"valid": "json"}"#.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                "{incomplete json".to_string(),
                r#"{"valid": "json"}"#.to_string()
            ])]
        );

        let results3 = buffer.add_line("more text".to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Incomplete(vec![
                "{incomplete json".to_string(),
                r#"{"valid": "json"}"#.to_string(),
                "more text".to_string()
            ])]
        );

        // Drain should extract valid JSON and flush the rest as text
        let drain_results = buffer.drain();
        assert_eq!(
            drain_results,
            vec![
                BufferResult::Text("{incomplete json".to_string()),
                BufferResult::Json(json!({"valid": "json"})),
                BufferResult::Text("more text".to_string())
            ]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_drain_only_invalid_content() {
        let mut buffer = LineBuffer::new(10);

        // Add only invalid JSON content
        let results1 = buffer.add_line("{invalid".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec!["{invalid".to_string()])]
        );

        let results2 = buffer.add_line("[also invalid".to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                "{invalid".to_string(),
                "[also invalid".to_string()
            ])]
        );

        // Drain should flush everything as text
        let drain_results = buffer.drain();
        assert_eq!(
            drain_results,
            vec![
                BufferResult::Text("{invalid".to_string()),
                BufferResult::Text("[also invalid".to_string())
            ]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_drain_valid_multiline_json() {
        let mut buffer = LineBuffer::new(10);

        // Add JSON-like line first to prevent immediate parsing
        let results0 = buffer.add_line("{Worf's honor code".to_string());
        assert_eq!(
            results0,
            vec![BufferResult::Incomplete(vec![
                "{Worf's honor code".to_string()
            ])]
        );

        // Add multi-line JSON that would be valid on its own
        let results1 = buffer.add_line("{".to_string());
        assert_eq!(
            results1,
            vec![BufferResult::Incomplete(vec![
                "{Worf's honor code".to_string(),
                "{".to_string()
            ])]
        );

        let results2 = buffer.add_line(r#"  "captain": "Sisko""#.to_string());
        assert_eq!(
            results2,
            vec![BufferResult::Incomplete(vec![
                "{Worf's honor code".to_string(),
                "{".to_string(),
                r#"  "captain": "Sisko""#.to_string()
            ])]
        );

        let results3 = buffer.add_line("}".to_string());
        assert_eq!(
            results3,
            vec![BufferResult::Incomplete(vec![
                "{Worf's honor code".to_string(),
                "{".to_string(),
                r#"  "captain": "Sisko""#.to_string(),
                "}".to_string()
            ])]
        );

        // Drain should extract the JSON-like text first, then the valid JSON
        let drain_results = buffer.drain();
        assert_eq!(
            drain_results,
            vec![
                BufferResult::Text("{Worf's honor code".to_string()),
                BufferResult::Json(json!({"captain": "Sisko"}))
            ]
        );
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn test_drain_empty_buffer() {
        let mut buffer = LineBuffer::new(10);

        // Drain on empty buffer should return empty results
        let drain_results = buffer.drain();
        assert_eq!(drain_results, vec![]);
        assert!(buffer.buffer.is_empty());
    }
}
