// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;

/// JSON Line Formatter - Process and format JSON data from streaming input
#[derive(Parser, Debug)]
#[command(version)]
pub struct JlifArgs {
    /// Maximum lines to buffer for multi-line JSON parsing
    #[arg(long, default_value = "10")]
    pub max_lines: usize,

    /// Regex pattern for filtering output
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Enable case-sensitive filtering
    #[arg(short = 's', long)]
    pub case_sensitive: bool,

    /// Show only JSON content, suppress non-JSON pass-through
    #[arg(short, long)]
    pub json_only: bool,

    /// Output JSON in compact format instead of pretty-printed
    #[arg(short, long)]
    pub compact: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Invert filter behavior - output everything that does NOT match
    #[arg(short = 'v', long)]
    pub invert_match: bool,
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_help_output() {
        let mut cmd = Command::cargo_bin("jlif").unwrap();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("JSON Line Formatter"))
            .stdout(predicate::str::contains("Usage:"));
    }

    #[test]
    fn test_version_output() {
        let mut cmd = Command::cargo_bin("jlif").unwrap();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::starts_with("jlif "))
            .stdout(predicate::str::is_match(r"^jlif \d+\.\d+\.\d+").unwrap());
    }

    #[test]
    fn test_invalid_argument_fails() {
        let mut cmd = Command::cargo_bin("jlif").unwrap();
        cmd.arg("--unknown-flag")
            .assert()
            .failure()
            .stderr(predicate::str::contains("unexpected argument"));
    }
}
