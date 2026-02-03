// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::buffer::BufferResult;
use enum_dispatch::enum_dispatch;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatterError {
    #[error("Invalid regex pattern '{pattern}': {source}")]
    InvalidRegex {
        pattern: String,
        #[source]
        source: regex::Error,
    },
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Cannot convert Incomplete buffer result to filter input")]
    IncompleteResult,
}

/// Input for filters - represents content that is ready for output filtering.
///
/// This enum only contains the outputtable variants from BufferResult (Json and Text),
/// ensuring that filters can never receive Incomplete states at compile time.
///
/// Uses borrowed references for zero-cost abstraction - no cloning or moving of
/// potentially large JSON values or strings during filtering operations.
#[derive(Debug)]
pub enum FilterInput<'a> {
    Json(&'a serde_json::Value),
    Text(&'a str),
}

impl<'a> TryFrom<&'a BufferResult> for FilterInput<'a> {
    type Error = ConversionError;

    fn try_from(result: &'a BufferResult) -> Result<Self, Self::Error> {
        match result {
            BufferResult::Json(value) => Ok(FilterInput::Json(value)),
            BufferResult::Text(text) => Ok(FilterInput::Text(text)),
            BufferResult::Incomplete(_) => Err(ConversionError::IncompleteResult),
        }
    }
}

/// Trait for filtering output content
///
/// Filters operate on FilterInput which provides type-safe access to only the
/// outputtable content types (Json and Text). The Incomplete variant is filtered
/// out during conversion, ensuring filters never need to handle internal processing states.
#[enum_dispatch]
pub trait Filter {
    /// Tests whether the given content matches the filter
    ///
    /// # Arguments
    /// * `input` - The content to test against the filter (Json or Text only)
    ///
    /// # Returns
    /// * `true` - Content matches the filter and should be output
    /// * `false` - Content does not match the filter and should be suppressed
    fn matches(&self, input: &FilterInput) -> bool;

    /// Returns true if this filter will potentially suppress content
    fn is_active(&self) -> bool;
}

/// No-op filter that passes all content through
#[derive(Debug, Clone)]
pub struct NoFilter;

impl Filter for NoFilter {
    fn matches(&self, _input: &FilterInput) -> bool {
        true
    }

    fn is_active(&self) -> bool {
        false
    }
}

/// Filter that inverts the result of an inner filter
#[derive(Debug)]
pub struct InvertFilter {
    inner_filter: Box<OutputFilter>,
}

impl InvertFilter {
    pub fn new(inner_filter: OutputFilter) -> Self {
        Self {
            inner_filter: Box::new(inner_filter),
        }
    }
}

impl Filter for InvertFilter {
    fn matches(&self, input: &FilterInput) -> bool {
        !self.inner_filter.matches(input)
    }

    fn is_active(&self) -> bool {
        self.inner_filter.is_active()
    }
}

/// Filter that only passes JSON content, applying an inner filter to JSON matches
#[derive(Debug)]
pub struct JsonOnlyFilter {
    inner_filter: Box<OutputFilter>,
}

impl JsonOnlyFilter {
    pub fn new(inner_filter: OutputFilter) -> Self {
        Self {
            inner_filter: Box::new(inner_filter),
        }
    }
}

impl Filter for JsonOnlyFilter {
    fn matches(&self, input: &FilterInput) -> bool {
        match input {
            FilterInput::Json(_) => self.inner_filter.matches(input),
            _ => false, // Suppress all non-JSON content (future-proof)
        }
    }

    fn is_active(&self) -> bool {
        true
    }
}

/// Regex-based filter with case sensitivity control
///
/// Converts both JSON and Text inputs to strings for regex matching.
/// JSON values are serialized to their compact string representation.
#[derive(Debug)]
pub struct RegexFilter {
    regex: Regex,
}

impl RegexFilter {
    pub fn new(pattern: String, case_sensitive: bool) -> Result<Self, FormatterError> {
        let regex_pattern = if case_sensitive {
            pattern.clone()
        } else {
            format!("(?i){}", pattern)
        };

        let regex = Regex::new(&regex_pattern)
            .map_err(|source| FormatterError::InvalidRegex { pattern, source })?;

        Ok(Self { regex })
    }
}

impl Filter for RegexFilter {
    fn matches(&self, input: &FilterInput) -> bool {
        let content = match input {
            FilterInput::Json(value) => {
                // Convert JSON to string for regex matching
                // Note: This does allocate a string, but only when filtering is active
                serde_json::to_string(value).unwrap_or_default()
            }
            FilterInput::Text(text) => (*text).to_string(),
        };

        self.regex.is_match(&content)
    }

    fn is_active(&self) -> bool {
        true
    }
}

/// Enum dispatch for different filter implementations
#[enum_dispatch(Filter)]
#[derive(Debug)]
pub enum OutputFilter {
    None(NoFilter),
    Regex(RegexFilter),
    JsonOnly(JsonOnlyFilter),
    Invert(InvertFilter),
}

impl OutputFilter {
    /// Creates a new OutputFilter from CLI arguments
    ///
    /// # Arguments
    /// * `pattern` - Optional regex pattern string. If None, returns NoFilter
    /// * `case_sensitive` - Whether the regex should be case sensitive
    /// * `json_only` - Whether to suppress all non-JSON output
    /// * `invert_match` - Whether to invert the filter behavior
    ///
    /// # Returns
    /// * `Ok(OutputFilter)` - Successfully created filter
    /// * `Err(FormatterError)` - Invalid regex pattern
    pub fn from_args(
        pattern: Option<String>,
        case_sensitive: bool,
        json_only: bool,
        invert_match: bool,
    ) -> Result<Self, FormatterError> {
        let base_filter = match pattern {
            Some(pattern_str) => {
                let regex_filter = RegexFilter::new(pattern_str, case_sensitive)?;
                OutputFilter::Regex(regex_filter)
            }
            None => OutputFilter::None(NoFilter),
        };

        let filter_with_json_only = if json_only {
            OutputFilter::JsonOnly(JsonOnlyFilter::new(base_filter))
        } else {
            base_filter
        };

        if invert_match {
            Ok(OutputFilter::Invert(InvertFilter::new(
                filter_with_json_only,
            )))
        } else {
            Ok(filter_with_json_only)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_input_conversion_json() {
        let buffer_result = BufferResult::Json(json!({"test": "value"}));
        let filter_input = FilterInput::try_from(&buffer_result).unwrap();

        match filter_input {
            FilterInput::Json(value) => {
                assert_eq!(value, &json!({"test": "value"}));
            }
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    fn test_filter_input_conversion_text() {
        let buffer_result = BufferResult::Text("test text".to_string());
        let filter_input = FilterInput::try_from(&buffer_result).unwrap();

        match filter_input {
            FilterInput::Text(text) => {
                assert_eq!(text, "test text");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_filter_input_conversion_incomplete_fails() {
        let buffer_result = BufferResult::Incomplete(vec!["incomplete".to_string()]);
        let result = FilterInput::try_from(&buffer_result);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::IncompleteResult => {} // Expected
        }
    }

    #[test]
    fn test_no_filter_passes_all() {
        let filter = OutputFilter::None(NoFilter);
        let json_value = json!({"test": "value"});
        let json_input = FilterInput::Json(&json_value);
        let text_input = FilterInput::Text("test text");

        assert!(filter.matches(&json_input));
        assert!(filter.matches(&text_input));
        assert!(!filter.is_active());
    }

    #[test]
    fn test_regex_filter_json_case_sensitive() {
        let filter = OutputFilter::Regex(RegexFilter::new("ERROR".to_string(), true).unwrap());

        let json_error_value = json!({"status": "ERROR", "message": "failed"});
        let json_lowercase_value = json!({"status": "error", "message": "failed"});
        let json_error = FilterInput::Json(&json_error_value);
        let json_lowercase = FilterInput::Json(&json_lowercase_value);

        assert!(filter.matches(&json_error));
        assert!(!filter.matches(&json_lowercase));
        assert!(filter.is_active());
    }

    #[test]
    fn test_regex_filter_text_case_insensitive() {
        let filter = OutputFilter::Regex(RegexFilter::new("error".to_string(), false).unwrap());

        let text_upper = FilterInput::Text("ERROR: something failed");
        let text_lower = FilterInput::Text("error: something failed");
        let text_mixed = FilterInput::Text("Error: something failed");
        let text_no_match = FilterInput::Text("info: everything ok");

        assert!(filter.matches(&text_upper));
        assert!(filter.matches(&text_lower));
        assert!(filter.matches(&text_mixed));
        assert!(!filter.matches(&text_no_match));
    }

    #[test]
    fn test_regex_filter_json_content_matching() {
        let filter = OutputFilter::Regex(RegexFilter::new("sisko".to_string(), false).unwrap());

        let json_match_value = json!({"captain": "Sisko", "station": "DS9"});
        let json_no_match_value = json!({"captain": "Picard", "ship": "Enterprise"});
        let json_match = FilterInput::Json(&json_match_value);
        let json_no_match = FilterInput::Json(&json_no_match_value);

        assert!(filter.matches(&json_match));
        assert!(!filter.matches(&json_no_match));
    }

    #[test]
    fn test_regex_filter_complex_patterns() {
        // Test JSON structure matching
        let filter = OutputFilter::Regex(
            RegexFilter::new(r#"\{"status"\s*:\s*"error""#.to_string(), false).unwrap(),
        );

        let json_error_value = json!({"status": "error", "message": "failed"});
        let json_ok_value = json!({"status": "ok", "message": "success"});
        let json_error = FilterInput::Json(&json_error_value);
        let json_ok = FilterInput::Json(&json_ok_value);

        assert!(filter.matches(&json_error));
        assert!(!filter.matches(&json_ok));
    }

    #[test]
    fn test_from_args_creates_correct_filter() {
        // No pattern creates NoFilter
        let no_filter = OutputFilter::from_args(None, false, false, false).unwrap();
        assert!(!no_filter.is_active());

        // Pattern creates RegexFilter
        let regex_filter =
            OutputFilter::from_args(Some("test".to_string()), true, false, false).unwrap();
        assert!(regex_filter.is_active());
    }

    #[test]
    fn test_from_args_invalid_regex() {
        let result = OutputFilter::from_args(Some("[".to_string()), false, false, false);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            FormatterError::InvalidRegex { pattern, .. } => {
                assert_eq!(pattern, "[");
            }
        }
    }

    #[test]
    fn test_json_only_filter_standalone() {
        let filter = OutputFilter::from_args(None, false, true, false).unwrap();

        let json_value = json!({"test": "data"});
        let json_input = FilterInput::Json(&json_value);
        let text_input = FilterInput::Text("plain text");

        assert!(filter.matches(&json_input));
        assert!(!filter.matches(&text_input));
        assert!(filter.is_active());
    }

    #[test]
    fn test_json_only_filter_with_regex() {
        let filter =
            OutputFilter::from_args(Some("error".to_string()), false, true, false).unwrap();

        let json_match_value = json!({"status": "error"});
        let json_no_match_value = json!({"status": "ok"});
        let json_match = FilterInput::Json(&json_match_value);
        let json_no_match = FilterInput::Json(&json_no_match_value);
        let text_match = FilterInput::Text("error occurred");

        // JSON that matches regex should pass
        assert!(filter.matches(&json_match));
        // JSON that doesn't match regex should not pass
        assert!(!filter.matches(&json_no_match));
        // Text should never pass, even if it matches regex
        assert!(!filter.matches(&text_match));
        assert!(filter.is_active());
    }

    #[test]
    fn test_from_args_combinations() {
        // No filter, no json-only
        let filter1 = OutputFilter::from_args(None, false, false, false).unwrap();
        assert!(!filter1.is_active());

        // Regex filter, no json-only
        let filter2 =
            OutputFilter::from_args(Some("test".to_string()), true, false, false).unwrap();
        assert!(filter2.is_active());

        // No filter, json-only
        let filter3 = OutputFilter::from_args(None, false, true, false).unwrap();
        assert!(filter3.is_active());

        // Regex filter + json-only
        let filter4 = OutputFilter::from_args(Some("test".to_string()), true, true, false).unwrap();
        assert!(filter4.is_active());
    }

    #[test]
    fn test_invert_filter_basic() {
        let filter =
            OutputFilter::from_args(Some("error".to_string()), false, false, true).unwrap();

        let json_match_value = json!({"status": "error"});
        let json_no_match_value = json!({"status": "ok"});
        let json_match = FilterInput::Json(&json_match_value);
        let json_no_match = FilterInput::Json(&json_no_match_value);
        let text_match = FilterInput::Text("error occurred");
        let text_no_match = FilterInput::Text("info message");

        // Invert should make matches fail and non-matches pass
        assert!(!filter.matches(&json_match)); // error matches pattern, so inverted = false
        assert!(filter.matches(&json_no_match)); // ok doesn't match pattern, so inverted = true
        assert!(!filter.matches(&text_match)); // error matches pattern, so inverted = false
        assert!(filter.matches(&text_no_match)); // info doesn't match pattern, so inverted = true
        assert!(filter.is_active());
    }

    #[test]
    fn test_invert_filter_with_no_pattern() {
        // Inverting NoFilter should still be active but invert everything
        let filter = OutputFilter::from_args(None, false, false, true).unwrap();

        let json_value = json!({"test": "data"});
        let json_input = FilterInput::Json(&json_value);
        let text_input = FilterInput::Text("plain text");

        // NoFilter normally passes everything, so inverted should block everything
        assert!(!filter.matches(&json_input));
        assert!(!filter.matches(&text_input));
        assert!(!filter.is_active()); // NoFilter is not active, so inverted NoFilter is also not active
    }

    #[test]
    fn test_invert_filter_with_json_only() {
        let filter = OutputFilter::from_args(Some("error".to_string()), false, true, true).unwrap();

        let json_match_value = json!({"status": "error"});
        let json_no_match_value = json!({"status": "ok"});
        let json_match = FilterInput::Json(&json_match_value);
        let json_no_match = FilterInput::Json(&json_no_match_value);
        let text_match = FilterInput::Text("error occurred");
        let text_no_match = FilterInput::Text("info message");

        // With json-only + invert: text is always suppressed, JSON is inverted
        assert!(!filter.matches(&json_match)); // JSON error matches, so inverted = false
        assert!(filter.matches(&json_no_match)); // JSON ok doesn't match, so inverted = true
        assert!(filter.matches(&text_match)); // Text is suppressed by json-only, so inverted = true
        assert!(filter.matches(&text_no_match)); // Text is suppressed by json-only, so inverted = true
        assert!(filter.is_active());
    }

    #[test]
    fn test_invert_filter_case_sensitivity() {
        let case_sensitive_filter =
            OutputFilter::from_args(Some("ERROR".to_string()), true, false, true).unwrap();
        let case_insensitive_filter =
            OutputFilter::from_args(Some("ERROR".to_string()), false, false, true).unwrap();

        let text_upper = FilterInput::Text("ERROR occurred");
        let text_lower = FilterInput::Text("error occurred");

        // Case sensitive: "ERROR" matches "ERROR" but not "error"
        assert!(!case_sensitive_filter.matches(&text_upper)); // matches, so inverted = false
        assert!(case_sensitive_filter.matches(&text_lower)); // doesn't match, so inverted = true

        // Case insensitive: "ERROR" matches both "ERROR" and "error"
        assert!(!case_insensitive_filter.matches(&text_upper)); // matches, so inverted = false
        assert!(!case_insensitive_filter.matches(&text_lower)); // matches, so inverted = false
    }

    #[test]
    fn test_invert_filter_combinations() {
        // Test all combinations with invert_match = true
        let filter1 = OutputFilter::from_args(None, false, false, true).unwrap();
        assert!(!filter1.is_active()); // NoFilter inverted is still not active

        let filter2 = OutputFilter::from_args(Some("test".to_string()), true, false, true).unwrap();
        assert!(filter2.is_active());

        let filter3 = OutputFilter::from_args(None, false, true, true).unwrap();
        assert!(filter3.is_active());

        let filter4 = OutputFilter::from_args(Some("test".to_string()), true, true, true).unwrap();
        assert!(filter4.is_active());
    }
}
