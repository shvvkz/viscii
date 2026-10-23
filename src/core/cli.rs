use clap::Parser;

#[derive(Parser)]
#[command(
    name = "viscii",
    version,
    about = "A small Rust CLI that plays a video in your terminal using ASCII characters"
)]
pub struct Cli {
    /// Path to the video file to play
    pub video_path: String,

    /// Enable inverted grayscale mode
    #[arg(short, long)]
    pub inverted_grayscale: bool,
}
