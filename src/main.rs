mod core;

use core::engine::extract_frames;

fn main() {
    loop {
        if let Err(e) = extract_frames("videos/bad_apple.mp4") {
            eprintln!("{}", e);
        }
    }
}
