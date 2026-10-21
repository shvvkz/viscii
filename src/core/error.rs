use std::fmt;
use std::fmt::Debug;

pub enum VisciiError {
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

impl fmt::Display for VisciiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisciiError::ErrorInput => {
                write!(f, "invalid video input")
            }
            VisciiError::ErrorStream => {
                write!(f, "Invalid video stream")
            }
            VisciiError::ErrorCreateDecoder => {
                write!(f, "Failded to create decoder context")
            }
            VisciiError::ErrorVideoDecoder => {
                write!(f, "Failed to create video decoder")
            }
            VisciiError::ErrorSendPacketDecoder => {
                write!(f, "Failed to send packet to decoder")
            }
            VisciiError::ErrorCreateScalor => {
                write!(f, "Failed to create a scalor")
            }
            VisciiError::ErrorConvertFrameRGB => {
                write!(f, "Failed to convert frame to RGB")
            }
            VisciiError::ErrorVideoDimensionsNotFound => {
                write!(f, "Failed to find video dimensions")
            }
            VisciiError::ErrorFrameRateNotFound => {
                write!(f, "Failed to find frame rate")
            }
        }
    }
}

impl Debug for VisciiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
