mod core;

use core::engine::extract_frames;

use crate::core::error::AsciiVideoError;

fn main() -> Result<(), AsciiVideoError> {
    loop {
        extract_frames("bad_apple")?;
    }
}
