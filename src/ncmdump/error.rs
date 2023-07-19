use std::io;
use thiserror::Error;

pub type DumpResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid file type")]
    FormatError,
    #[error("Cannot read key length")]
    KeyLengthError,
    #[error("Cannot read key")]
    KeyLoadError,
    #[error("Cannot read info length")]
    InfoLengthError,
    #[error("Cannot read info")]
    InfoLoadError,
    #[error("Cannot decode info")]
    InfoDecodeError,
    #[error("Cannot read image length")]
    ImageLengthError,
    #[error("Cannot read image")]
    ImageLoadError,
    #[error("Cannot guess image format")]
    ImageFormatError,
    #[error("Unsupported image format")]
    ImageUnsupportedError,

    #[error("Cannot decrypt the key")]
    KeyDecryptError,

    #[error("Cannot build the tag: {0}")]
    TagBuildError(String),
    #[error("Cannot write the tag: {0}")]
    TagWritedError(String),

    #[error("IO error: {0}")]
    IO(String),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

#[cfg(feature = "tag")]
impl From<id3::Error> for Error {
    fn from(value: id3::Error) -> Self {
        Self::TagBuildError(value.to_string())
    }
}

#[cfg(feature = "tag")]
impl From<metaflac::Error> for Error {
    fn from(value: metaflac::Error) -> Self {
        Self::TagBuildError(value.to_string())
    }
}

#[cfg(feature = "tag")]
impl From<audiotags::Error> for Error {
    fn from(value: audiotags::Error) -> Self {
        Self::TagWritedError(value.to_string())
    }
}
