mod buffer;
mod cli;
mod filter;
mod processor;

use anyhow::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
use buffer::LineBuffer;
use clap::Parser;
use cli::JlifArgs;
use filter::OutputFilter;
use processor::StreamProcessor;
use std::io;

fn main() -> Result<()> {
    let args = JlifArgs::parse();

    // Create filter from CLI arguments
    let filter = OutputFilter::from_args(args.filter, args.case_sensitive, args.json_only)
        .map_err(|e| anyhow::anyhow!("Filter error: {}", e))?;

    // Create LineBuffer with user-specified max_lines
    let line_buffer = LineBuffer::new(args.max_lines);

    // Create StreamProcessor with stdin, stdout, buffer, and filter
    let mut stream_processor = StreamProcessor::new(io::stdin(), io::stdout(), line_buffer, filter);

    // Process the stream
    stream_processor.process()?;

    Ok(())
}
