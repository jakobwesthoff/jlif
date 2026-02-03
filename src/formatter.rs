// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::Result;
use colored_json::ColoredFormatter;
use enum_dispatch::enum_dispatch;
use serde_json::ser::{CompactFormatter, PrettyFormatter};

/// JSON formatter variants that handle both colored and non-colored output.
///
/// This enum uses enum_dispatch for zero-cost static dispatch as an alternative to trait objects.
/// The serde_json::ser::Formatter trait is not dyn-compatible due to its generic methods,
/// making trait objects impossible. Enum dispatch provides the same polymorphic behavior
/// with compile-time dispatch and better performance.
///
/// Uses the appropriate serde_json convenience functions (to_string, to_string_pretty)
/// and colored_json functions for colored output, avoiding the complexity of manual
/// serializer management while maintaining good performance.
#[enum_dispatch(Formatter)]
pub enum JsonFormatter {
    ColoredCompact(ColoredCompactFormatter),
    ColoredPretty(ColoredPrettyFormatter),
    PlainCompact(PlainCompactFormatter),
    PlainPretty(PlainPrettyFormatter),
}

impl JsonFormatter {
    /// Creates the appropriate JSON formatter from CLI arguments
    pub fn from_args(compact: bool, no_color: bool) -> Self {
        match (compact, no_color) {
            (true, true) => JsonFormatter::PlainCompact(PlainCompactFormatter::new()),
            (true, false) => JsonFormatter::ColoredCompact(ColoredCompactFormatter::new()),
            (false, true) => JsonFormatter::PlainPretty(PlainPrettyFormatter::new()),
            (false, false) => JsonFormatter::ColoredPretty(ColoredPrettyFormatter::new()),
        }
    }
}

#[enum_dispatch]
pub trait Formatter {
    fn format_json(&self, value: &serde_json::Value) -> Result<String>;
}

/// Colored compact JSON formatter using colored_json with CompactFormatter
pub struct ColoredCompactFormatter;

impl ColoredCompactFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for ColoredCompactFormatter {
    fn format_json(&self, value: &serde_json::Value) -> Result<String> {
        let formatter = ColoredFormatter::new(CompactFormatter {});
        Ok(formatter.to_colored_json_auto(value)?)
    }
}

/// Colored pretty-printed JSON formatter using colored_json with PrettyFormatter
pub struct ColoredPrettyFormatter;

impl ColoredPrettyFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for ColoredPrettyFormatter {
    fn format_json(&self, value: &serde_json::Value) -> Result<String> {
        let formatter = ColoredFormatter::new(PrettyFormatter::new());
        Ok(formatter.to_colored_json_auto(value)?)
    }
}

/// Plain compact JSON formatter using serde_json::to_string
pub struct PlainCompactFormatter;

impl PlainCompactFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for PlainCompactFormatter {
    fn format_json(&self, value: &serde_json::Value) -> Result<String> {
        Ok(serde_json::to_string(value)?)
    }
}

/// Plain pretty-printed JSON formatter using serde_json::to_string_pretty
pub struct PlainPrettyFormatter;

impl PlainPrettyFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for PlainPrettyFormatter {
    fn format_json(&self, value: &serde_json::Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }
}
