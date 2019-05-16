use failure::Fail;
use std::fmt::Debug;

#[derive(Debug, Fail)]
pub enum CargoPlayError {
    #[fail(display = "IO error: {:?}", _0)]
    IOError(std::io::Error),

    #[fail(display = "Parsing error: {:?}", _0)]
    ParseError(String),
}

impl From<std::io::Error> for CargoPlayError {
    fn from(value: std::io::Error) -> Self {
        CargoPlayError::IOError(value)
    }
}

impl CargoPlayError {
    pub fn from_serde<T: Debug>(value: T) -> Self {
        Self::ParseError(format!("{:?}", value))
    }
}
