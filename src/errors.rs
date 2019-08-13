use failure::Fail;
use std::fmt::Debug;

#[derive(Debug, Fail)]
pub enum CargoPlayError {
    #[fail(display = "IO error: {:?}", _0)]
    IOError(std::io::Error),

    #[fail(display = "Parsing error: {:?}", _0)]
    ParseError(String),

    #[fail(display = "Unable to compute relative path of {:?}", _0)]
    DiffPathError(std::path::PathBuf),

    #[fail(display = "Unexpected edition {:?}. Edition must be 2015/2018.", _0)]
    InvalidEdition(String),

    #[fail(display = "Path already exists at {:?}", _0)]
    PathExistError(std::path::PathBuf),

    /// Helper error kind only exists for development purpose.
    #[fail(display = "{:?}", _0)]
    _Message(String),
}

impl From<std::io::Error> for CargoPlayError {
    fn from(value: std::io::Error) -> Self {
        CargoPlayError::IOError(value)
    }
}

impl CargoPlayError {
    pub fn from_serde<T: Debug>(value: T) -> Self {
        CargoPlayError::ParseError(format!("{:?}", value))
    }

    pub fn _message<T: Into<String>>(value: T) -> Self {
        CargoPlayError::_Message(value.into())
    }
}
