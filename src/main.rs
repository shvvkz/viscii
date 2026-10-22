mod core;

use core::cli::Cli;
use core::engine::extract_frames;
use core::error::VisciiError;

use clap::Parser;

fn main() -> Result<(), VisciiError> {
    let args = Cli::parse();
    loop {
        extract_frames(&args)?;
    }
}
