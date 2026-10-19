use super::error::AsciiVideoError;
use crossterm::terminal;
use image::{DynamicImage, RgbImage};

const CHAR_RATIO: f64 = 0.5;
const ASCII_PIXELS: &str = " .,:;irsXA253hMHGS#9B&@";

fn optimal_resolution(video_width: u32, video_height: u32) -> Result<(u32, u32), AsciiVideoError> {
    let (cols, _) = terminal::size().map_err(|_| AsciiVideoError::ErrorVideoDimensionsNotFound)?;

    if video_width == 0 || video_height == 0 {
        return Err(AsciiVideoError::ErrorVideoDimensionsNotFound);
    }

    let aspect_ratio = video_width as f64 / video_height as f64;
    let ideal_height = (cols as f64 / aspect_ratio * CHAR_RATIO) as u32;

    Ok((cols as u32, ideal_height))
}

fn into_dynamic_image(rgb_frame: &ffmpeg_next::frame::Video) -> DynamicImage {
    let mut buffer: RgbImage = image::ImageBuffer::new(rgb_frame.width(), rgb_frame.height());

    for (x, y, pixel) in buffer.enumerate_pixels_mut() {
        let data = rgb_frame.data(0);
        let stride = rgb_frame.stride(0);
        let offset = y as usize * stride + x as usize * 3;
        *pixel = image::Rgb([data[offset], data[offset + 1], data[offset + 2]]);
    }

    image::DynamicImage::ImageRgb8(buffer)
}

struct VideoInfo {
    width: u32,
    height: u32,
    frame_rate: f64,
}

fn open_video(path: &str) -> Result<ffmpeg_next::format::context::Input, AsciiVideoError> {
    ffmpeg_next::init().map_err(|_| AsciiVideoError::ErrorInit)?;

    ffmpeg_next::format::input(&std::path::Path::new(path)).map_err(|_| AsciiVideoError::ErrorInput)
}

fn create_decoder(
    input: &ffmpeg_next::format::context::Input,
) -> Result<(usize, ffmpeg_next::decoder::Video), AsciiVideoError> {
    let stream = input
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(AsciiVideoError::ErrorStream)?;

    let stream_index = stream.index();

    let context = ffmpeg_next::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|_| AsciiVideoError::ErrorCreateDecoder)?;

    let decoder = context
        .decoder()
        .video()
        .map_err(|_| AsciiVideoError::ErrorVideoDecoder)?;

    Ok((stream_index, decoder))
}

fn get_video_info(decoder: &ffmpeg_next::decoder::Video) -> Result<VideoInfo, AsciiVideoError> {
    let (width, height) = optimal_resolution(decoder.width(), decoder.height())
        .map_err(|_| AsciiVideoError::ErrorInit)?;

    let rate = decoder
        .frame_rate()
        .ok_or(AsciiVideoError::ErrorFrameRateNotFound)?;

    if rate.denominator() == 0 || rate.numerator() <= 0 {
        return Err(AsciiVideoError::ErrorFrameRateNotFound);
    }

    let frame_rate = rate.numerator() as f64 / rate.denominator() as f64;

    Ok(VideoInfo {
        width,
        height,
        frame_rate,
    })
}

fn create_scaler(
    decoder: &ffmpeg_next::decoder::Video,
    info: &VideoInfo,
) -> Result<ffmpeg_next::software::scaling::Context, AsciiVideoError> {
    ffmpeg_next::software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg_next::format::Pixel::RGB24,
        info.width,
        info.height,
        ffmpeg_next::software::scaling::Flags::BILINEAR,
    )
    .map_err(|_| AsciiVideoError::ErrorCreateScalor)
}

fn pixel_to_ascii(r: u8, g: u8, b: u8) -> char {
    let gray = (0.3 * r as f32 + 0.6 * g as f32 + 0.1 * b as f32) as u8;
    let index = (gray as usize * (ASCII_PIXELS.len() - 1)) / 255;
    ASCII_PIXELS.as_bytes()[index] as char
}

fn rgb_to_ascii(rgb: &RgbImage) -> String {
    let width = rgb.width() as usize;
    let mut ascii = String::new();

    for (i, pixel) in rgb.as_raw().chunks_exact(3).enumerate() {
        let c = pixel_to_ascii(pixel[0], pixel[1], pixel[2]);

        ascii.push(c);

        if (i + 1) % width == 0 {
            ascii.push('\n');
        }
    }

    ascii
}

fn process_image(
    decoded: &ffmpeg_next::frame::Video,
    scaler: &mut ffmpeg_next::software::scaling::Context,
) -> Result<String, AsciiVideoError> {
    let mut rgb_frame = ffmpeg_next::frame::Video::empty();

    scaler
        .run(decoded, &mut rgb_frame)
        .map_err(|_| AsciiVideoError::ErrorConvertFrameRGB)?;

    let image = into_dynamic_image(&rgb_frame);

    let rgb = image.to_rgb8();

    Ok(rgb_to_ascii(&rgb))
}

fn display_image(ascii: &str, frame_rate: f64) {
    print!("{}", ascii);

    std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / frame_rate));
}

fn receive_frames(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    frame_rate: f64,
) -> Result<(), AsciiVideoError> {
    let mut frame = ffmpeg_next::frame::Video::empty();

    while decoder.receive_frame(&mut frame).is_ok() {
        let ascii = process_image(&frame, scaler)?;

        display_image(&ascii, frame_rate);
    }

    Ok(())
}

fn process_frames(
    input: &mut ffmpeg_next::format::context::Input,
    stream_index: usize,
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    frame_rate: f64,
) -> Result<(), AsciiVideoError> {
    for (stream, packet) in input.packets() {
        if stream.index() != stream_index {
            continue;
        }

        decoder
            .send_packet(&packet)
            .map_err(|_| AsciiVideoError::ErrorSendPacketDecoder)?;

        receive_frames(decoder, scaler, frame_rate)?;
    }

    decoder
        .send_eof()
        .map_err(|_| AsciiVideoError::ErrorSendPacketDecoder)?;

    receive_frames(decoder, scaler, frame_rate)?;

    Ok(())
}

pub fn extract_frames(video_path: &str) -> Result<(), AsciiVideoError> {
    let mut input = open_video(video_path)?;

    let (stream_index, mut decoder) = create_decoder(&input)?;

    let info = get_video_info(&decoder)?;

    let mut scaler = create_scaler(&decoder, &info)?;

    process_frames(
        &mut input,
        stream_index,
        &mut decoder,
        &mut scaler,
        info.frame_rate,
    )?;

    Ok(())
}
