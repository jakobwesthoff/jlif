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
    #[arg(short, long)]
    pub case_sensitive: bool,

    /// Show only JSON content, suppress non-JSON pass-through
    #[arg(short, long)]
    pub json_only: bool,
}
