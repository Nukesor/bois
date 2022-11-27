use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while building path: {}", .0)]
    InvalidPath(String),

    #[error("Error while reading configuration:\n{}", .0)]
    ConfigDeserialization(String),

    #[error("Some error occurred. {}", .0)]
    Generic(String),

    #[error("I/O error while {}:\n{}", .0, .1)]
    IoError(String, std::io::Error),

    #[error("Unexpected I/O error:\n{}", .0)]
    RawIoError(#[from] std::io::Error),

    #[error("I/O error at path {:?} while {}:\n{}", .0, .1, .2)]
    IoPathError(PathBuf, &'static str, std::io::Error),
}
