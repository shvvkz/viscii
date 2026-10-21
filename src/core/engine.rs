use super::error::VisciiError;
use crossterm::terminal;
use image::{DynamicImage, RgbImage};

const CHAR_RATIO: f64 = 0.5;
const ASCII_PIXELS: &str = " .,:;irsXA253hMHGS#9B&@";
// const ASCII_PIXELS: &str = "@&B9#SGHMh352AXssri;:,. ";

struct VideoInfo {
    width: u32,
    height: u32,
    frame_rate: f64,
}

struct AsciiPadding {
    up: String,
    right: String,
    down: String,
    left: String,
}

fn optimal_resolution(video_width: u32, video_height: u32) -> Result<(u32, u32), VisciiError> {
    let (cols, rows) = terminal::size().map_err(|_| VisciiError::ErrorVideoDimensionsNotFound)?;

    if video_width == 0 || video_height == 0 || cols == 0 || rows == 0 {
        return Err(VisciiError::ErrorVideoDimensionsNotFound);
    }

    let aspect_ratio = video_width as f64 / video_height as f64;
    let ideal_height = cols as f64 / aspect_ratio * CHAR_RATIO;

    let (width, height) = if ideal_height > rows as f64 {
        let height = rows as u32;
        let width = (height as f64 * aspect_ratio / CHAR_RATIO) as u32;

        (width, height)
    } else {
        (cols as u32, ideal_height as u32)
    };

    let width = width.max(1);
    let height = height.max(1);

    Ok((width, height))
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

fn open_video(path: &str) -> Result<ffmpeg_next::format::context::Input, VisciiError> {
    ffmpeg_next::init().map_err(|_| VisciiError::ErrorInput)?;

    ffmpeg_next::format::input(&std::path::Path::new(path)).map_err(|_| VisciiError::ErrorInput)
}

fn create_decoder(
    input: &ffmpeg_next::format::context::Input,
) -> Result<(usize, ffmpeg_next::decoder::Video), VisciiError> {
    let stream = input
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or(VisciiError::ErrorStream)?;

    let stream_index = stream.index();

    let context = ffmpeg_next::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|_| VisciiError::ErrorCreateDecoder)?;

    let decoder = context
        .decoder()
        .video()
        .map_err(|_| VisciiError::ErrorVideoDecoder)?;

    Ok((stream_index, decoder))
}

fn get_video_info(decoder: &ffmpeg_next::decoder::Video) -> Result<VideoInfo, VisciiError> {
    let (width, height) = optimal_resolution(decoder.width(), decoder.height())?;

    let rate = decoder
        .frame_rate()
        .ok_or(VisciiError::ErrorFrameRateNotFound)?;

    if rate.denominator() == 0 || rate.numerator() <= 0 {
        return Err(VisciiError::ErrorFrameRateNotFound);
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
) -> Result<ffmpeg_next::software::scaling::Context, VisciiError> {
    ffmpeg_next::software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg_next::format::Pixel::RGB24,
        info.width,
        info.height,
        ffmpeg_next::software::scaling::Flags::BILINEAR,
    )
    .map_err(|_| VisciiError::ErrorCreateScalor)
}

fn pixel_to_ascii(r: u8, g: u8, b: u8) -> char {
    let gray = (0.3 * r as f32 + 0.6 * g as f32 + 0.1 * b as f32) as u8;
    let index = (gray as usize * (ASCII_PIXELS.len() - 1)) / 255;
    ASCII_PIXELS.as_bytes()[index] as char
}

fn rgb_to_ascii(rgb: &RgbImage, ascii_padding: &AsciiPadding) -> String {
    let width = rgb.width() as usize;
    let mut ascii = String::new();
    ascii.push_str(&ascii_padding.up);
    for (i, pixel) in rgb.as_raw().as_chunks::<3>().0.iter().enumerate() {
        let c = pixel_to_ascii(pixel[0], pixel[1], pixel[2]);

        ascii.push(c);

        if (i + 1) % width == 0 {
            ascii.push_str(&ascii_padding.right);
            ascii.push('\n');
            ascii.push_str(&ascii_padding.left);
        }
    }
    ascii.push_str(&ascii_padding.down);
    ascii
}

fn process_image(
    decoded: &ffmpeg_next::frame::Video,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    ascii_padding: &AsciiPadding,
) -> Result<String, VisciiError> {
    let mut rgb_frame = ffmpeg_next::frame::Video::empty();

    scaler
        .run(decoded, &mut rgb_frame)
        .map_err(|_| VisciiError::ErrorConvertFrameRGB)?;

    let image = into_dynamic_image(&rgb_frame);

    let rgb = image.to_rgb8();

    Ok(rgb_to_ascii(&rgb, ascii_padding))
}

fn display_image(ascii: &str, frame_rate: f64) {
    print!("{}", ascii);

    std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / frame_rate));
}

fn receive_frames(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    video_info: &VideoInfo,
    ascii_padding: &AsciiPadding,
) -> Result<(), VisciiError> {
    let mut frame = ffmpeg_next::frame::Video::empty();

    while decoder.receive_frame(&mut frame).is_ok() {
        let ascii = process_image(&frame, scaler, ascii_padding)?;

        display_image(&ascii, video_info.frame_rate);
    }

    Ok(())
}

fn process_frames(
    input: &mut ffmpeg_next::format::context::Input,
    stream_index: usize,
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut ffmpeg_next::software::scaling::Context,
    video_info: &VideoInfo,
    ascii_padding: &AsciiPadding,
) -> Result<(), VisciiError> {
    for (stream, packet) in input.packets() {
        if stream.index() != stream_index {
            continue;
        }

        decoder
            .send_packet(&packet)
            .map_err(|_| VisciiError::ErrorSendPacketDecoder)?;

        receive_frames(decoder, scaler, video_info, ascii_padding)?;
    }

    decoder
        .send_eof()
        .map_err(|_| VisciiError::ErrorSendPacketDecoder)?;

    receive_frames(decoder, scaler, video_info, ascii_padding)?;

    Ok(())
}

fn get_ascii_padding(info: &VideoInfo) -> AsciiPadding {
    let mut padding_height_up = String::new();
    let mut padding_height_down = String::new();
    let mut padding_width_left = String::new();
    let mut padding_width_right = String::new();

    let (cols, rows) = terminal::size().unwrap();

    let nb_padding_height = (rows - info.height as u16) / 2;
    if nb_padding_height != 0 {
        let nb_padding_height_up = if nb_padding_height.is_multiple_of(2) {
            nb_padding_height - 1
        } else {
            nb_padding_height
        };
        padding_height_up =
            std::iter::repeat_n("\n", nb_padding_height_up as usize).collect::<String>();
        padding_height_down =
            std::iter::repeat_n("\n", nb_padding_height as usize).collect::<String>();
    }

    let nb_padding_width = (cols - info.width as u16) / 2;
    if nb_padding_width != 0 {
        let nb_padding_width_left = if nb_padding_width.is_multiple_of(2) {
            nb_padding_width - 1
        } else {
            nb_padding_width
        };
        padding_width_left =
            std::iter::repeat_n(" ", nb_padding_width_left as usize).collect::<String>();
        padding_width_right =
            std::iter::repeat_n(" ", nb_padding_width as usize).collect::<String>();
    }

    AsciiPadding {
        up: padding_height_up,
        right: padding_width_right,
        down: padding_height_down,
        left: padding_width_left,
    }
}

pub fn extract_frames(video_path: &str) -> Result<(), VisciiError> {
    let mut input = open_video(video_path)?;

    let (stream_index, mut decoder) = create_decoder(&input)?;

    let info = get_video_info(&decoder)?;

    let mut scaler = create_scaler(&decoder, &info)?;

    let ascii_padding = get_ascii_padding(&info);

    process_frames(
        &mut input,
        stream_index,
        &mut decoder,
        &mut scaler,
        &info,
        &ascii_padding,
    )?;

    Ok(())
}
