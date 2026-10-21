mod core;

use core::engine::extract_frames;
use core::error::VisciiError;

fn main() -> Result<(), VisciiError> {
    loop {
        extract_frames("videos/bad_apple.mp4")?;
    }
}
