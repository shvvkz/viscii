use clap::Parser;

#[derive(Parser)]
#[command(
    name = "viscii",
    version,
    about = "A small Rust CLI that plays a video in your terminal using ASCII characters"
)]
pub struct Cli {
    pub video_path: String,
}
