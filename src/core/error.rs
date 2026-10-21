use std::fmt;
use std::fmt::Debug;

pub enum AsciiVideoError {
    ErrorInput,
    ErrorStream,
    ErrorCreateDecoder,
    ErrorVideoDecoder,
    ErrorSendPacketDecoder,
    ErrorCreateScalor,
    ErrorConvertFrameRGB,
    ErrorVideoDimensionsNotFound,
    ErrorFrameRateNotFound,
}

impl fmt::Display for AsciiVideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsciiVideoError::ErrorInput => {
                write!(f, "invalid video input")
            }
            AsciiVideoError::ErrorStream => {
                write!(f, "Invalid video stream")
            }
            AsciiVideoError::ErrorCreateDecoder => {
                write!(f, "Failded to create decoder context")
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
            AsciiVideoError::ErrorVideoDimensionsNotFound => {
                write!(f, "Failed to find video dimensions")
            }
            AsciiVideoError::ErrorFrameRateNotFound => {
                write!(f, "Failed to find frame rate")
            }
        }
    }
}

impl Debug for AsciiVideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
