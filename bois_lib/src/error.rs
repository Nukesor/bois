use std::path::PathBuf;

use file_owner::FileOwnerError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Some error occurred. {}", .0)]
    Generic(String),

    #[error("Couldn't find file {} in directory: {:?}", .0, .1)]
    FileNotFound(String, PathBuf),

    #[error("I/O error while {}:\n{}", .0, .1)]
    Io(String, std::io::Error),

    #[error("Unexpected I/O error:\n{}", .0)]
    RawIo(#[from] std::io::Error),

    #[error("I/O error at path {:?} while {}:\n{}", .0, .1, .2)]
    IoPath(PathBuf, &'static str, std::io::Error),

    // Same as the IoPathError, but with a String. Less ergonomic
    #[error("I/O error at path {:?} while {}:\n{}", .0, .1, .2)]
    IoPathString(PathBuf, String, std::io::Error),

    #[error("I/O error at path {:?} while {}:\n{}", .0, .1, .2)]
    FileOwnership(PathBuf, &'static str, FileOwnerError),

    #[error("Error while running process '{}'\nError:\n{}", .0, .1)]
    Process(&'static str, std::io::Error),

    #[error("Deserialization error for file {:?}:\n {}", .0, .1)]
    Deserialization(PathBuf, serde_yaml::Error),

    #[error("Permission error while {:?}", .0)]
    Permission(&'static str),
}
