mod cli;

use clap::Parser;
use cli::JlifArgs;

fn main() {
    let args = JlifArgs::parse();

    println!("Max lines: {}", args.max_lines);
    println!("Filter: {:?}", args.filter);
    println!("Case sensitive: {}", args.case_sensitive);
    println!("JSON only: {}", args.json_only);
}
