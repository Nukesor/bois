use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while reading configuration:\n{}", .0)]
    ConfigDeserialization(String),

    #[error("Some error occurred. {}", .0)]
    Generic(String),

    #[error("Couldn't find file {} in directory: {:?}", .0, .1)]
    FileNotFound(String, PathBuf),

    #[error("I/O error while {}:\n{}", .0, .1)]
    IoError(String, std::io::Error),

    #[error("Unexpected I/O error:\n{}", .0)]
    RawIoError(#[from] std::io::Error),

    #[error("I/O error at path {:?} while {}:\n{}", .0, .1, .2)]
    IoPathError(PathBuf, &'static str, std::io::Error),

    #[error("Deserialization error for file {:?}:\n {}", .0, .1)]
    DeserializationError(PathBuf, serde_yaml::Error),
}
