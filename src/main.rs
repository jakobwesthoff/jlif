mod buffer;
mod cli;
mod processor;

use anyhow::Result;
use buffer::LineBuffer;
use clap::Parser;
use cli::JlifArgs;
use processor::StreamProcessor;
use std::io;

fn main() -> Result<()> {
    let args = JlifArgs::parse();

    // Create LineBuffer with user-specified max_lines
    let line_buffer = LineBuffer::new(args.max_lines);

    // Create StreamProcessor with stdin and stdout
    let mut stream_processor = StreamProcessor::new(io::stdin(), io::stdout(), line_buffer);

    // Process the stream
    stream_processor.process()?;

    Ok(())
}
