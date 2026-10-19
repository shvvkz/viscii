use ffmpeg_next;
use image::{DynamicImage, RgbImage};
use std::path::Path;
use std::fmt;

enum AsciiVideoError{
    ErrorInit,
    ErrorInput,
    ErrorStream,
    ErrorCreateDecoder,
    ErrorVideoDecoder,
    ErrorSendPacketDecoder,
    ErrorCreateScalor,
    ErrorConvertFrameRGB
}

impl fmt::Display for AsciiVideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsciiVideoError::ErrorInit => {
                write!(f, "Invalid Init")
            }
            AsciiVideoError::ErrorInput => {
                write!(f, "invalid Input")
            }
            AsciiVideoError::ErrorStream => {
                write!(f, "Invalid Stream")
            }
            AsciiVideoError::ErrorCreateDecoder => {
                write!(f, "Failded to created decoder context")
            }
            AsciiVideoError::ErrorVideoDecoder => {
                write!(f, "Failed to create video decoder")
            }
            AsciiVideoError::ErrorSendPacketDecoder => {
                write!(f, "Failed to send packet to decoder")
            }
            AsciiVideoError::ErrorCreateScalor => {
                write!(f, "Failed to create a scalor")
            }
            AsciiVideoError::ErrorConvertFrameRGB => {
                write!(f, "Failed to convert frame to RGB")
            }
        }
    }
}

fn into_dynamic_image(rgb_frame: &ffmpeg_next::frame::Video) -> DynamicImage {

    let mut buffer: RgbImage =
        image::ImageBuffer::new(rgb_frame.width() as u32, rgb_frame.height() as u32);

    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
        let data = rgb_frame.data(0);
        let stride = rgb_frame.stride(0) as usize;
        let offset = y as usize * stride + x as usize * 3;
        *pixel = image::Rgb([
            data[offset],
            data[offset + 1],
            data[offset + 2],
        ]);
    }

    image::DynamicImage::ImageRgb8(buffer)
}

fn extract_frames(video_path: &str) -> Result<(), AsciiVideoError> {
    ffmpeg_next::init().map_err(|_| AsciiVideoError::ErrorInit)?;

    let mut input = ffmpeg_next::format::input(&Path::new(video_path)).map_err(|_| AsciiVideoError::ErrorInput)?;

    let stream = input
        .streams()
        .best(ffmpeg_next::media::Type::Video).ok_or(AsciiVideoError::ErrorStream)?;

    let stream_index = stream.index();

    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(stream.parameters()).map_err(|_| AsciiVideoError::ErrorCreateDecoder)?;
    let mut decoder = context_decoder.decoder().video().map_err(|_| AsciiVideoError::ErrorVideoDecoder)?;

    let mut frame_index = 0;
    
    let mut decoded_frame = ffmpeg_next::frame::Video::empty();
    let mut rgb_frame = ffmpeg_next::frame::Video::empty();
 
    // Get video dimensions
    let width = decoder.width();
    let height = decoder.height();

    // Create scaler for conversion to RGB
    let mut scaler = ffmpeg_next::software::scaling::Context::get(
        decoder.format(),
        width,
        height,
        ffmpeg_next::format::Pixel::RGB24,
        width,
        height,
        ffmpeg_next::software::scaling::Flags::BILINEAR,
    ).map_err(|_| AsciiVideoError::ErrorCreateScalor)?;

    
    for (stream, packet) in input.packets() {
        if stream.index() == stream_index {
            decoder.send_packet(&packet)
                .map_err(|_| AsciiVideoError::ErrorSendPacketDecoder)?;
            
            // Try to receive a frame
            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                // Convert FFmpeg frame to an image buffer
                // Convert to RGB
                scaler.run(&decoded_frame, &mut rgb_frame)
                    .map_err(|_| AsciiVideoError::ErrorConvertFrameRGB)?;
                
                let dynamic_image = into_dynamic_image(&rgb_frame);

                let rgb = dynamic_image.to_rgb8();
                let grayscale: Vec<u8> = rgb.into_raw()
                    .chunks_exact(3)
                    .map(|pixel| {
                        let r = pixel[0] as f32;
                        let g = pixel[1] as f32;
                        let b = pixel[2] as f32;

                        (0.299 * r + 0.587 * g + 0.114 * b) as u8
                    })
                    .collect();
                println!("{:?}, len: {}", grayscale, grayscale.len());
            }
        }
    }
    Ok(())
}

fn main() {
    let _ = extract_frames("video.mp4");
}
