use std::io;
use std::sync::OnceLock;
use anyhow::Error;
use thiserror::Error;

pub(crate) mod decoder;

pub(crate) mod encoder;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Packet codec error: {0}")]
    CodecError(Error),
    #[error("Tea_key length is invalid")]
    InvalidTeaKey,
    #[error("IO error")]
    IoError
}

pub(crate) static mut LAST_PACKET_TIME: i64 = 0i64;

impl From<io::Error> for CodecError {
    fn from(value: io::Error) -> Self {
        CodecError::CodecError(Error::new(value))
    }
}

pub(crate) fn enable_print_codec_logs() -> &'static bool {
    static ENABLE_PRINT_CODEC_LOG: OnceLock<bool> = OnceLock::new();
    ENABLE_PRINT_CODEC_LOG.get_or_init(|| {
        std::env::var("ENABLE_PRINT_CODEC_LOG")
            .map_or(false, |v| v == "1")
    })
}