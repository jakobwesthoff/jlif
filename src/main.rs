// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod buffer;
mod cli;
mod filter;
mod formatter;
mod processor;

use anyhow::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
use buffer::LineBuffer;
use clap::Parser;
use cli::JlifArgs;
use filter::OutputFilter;
use formatter::JsonFormatter;
use processor::StreamProcessor;
use std::io;

fn main() -> Result<()> {
    let args = JlifArgs::parse();

    // Create filter from CLI arguments
    let filter = OutputFilter::from_args(
        args.filter,
        args.case_sensitive,
        args.json_only,
        args.invert_match,
    )
    .map_err(|e| anyhow::anyhow!("Filter error: {}", e))?;

    // Create LineBuffer with user-specified max_lines
    let line_buffer = LineBuffer::new(args.max_lines);

    // Create the appropriate JSON formatter based on flags
    let json_formatter = JsonFormatter::from_args(args.compact, args.no_color);

    // Create StreamProcessor with stdin, stdout, buffer, filter, and formatter
    let mut stream_processor = StreamProcessor::new(
        io::stdin(),
        io::stdout(),
        line_buffer,
        filter,
        json_formatter,
    );

    // Process the stream
    stream_processor.process()?;

    Ok(())
}
