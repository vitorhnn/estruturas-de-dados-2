use std::fs::File;
use std::io;
use std::marker::Sized;
use std::string::FromUtf8Error;

use chrono::ParseError;

pub trait Serializable where Self: Sized {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError>;

    fn deserialize(file: &mut File) -> Result<Self, SerializeError>;
}

#[derive(Debug)]
pub enum SerializeError {
    Io(io::Error),
    Utf8(FromUtf8Error),
    Chrono(ParseError),
}

impl From<io::Error> for SerializeError {
    fn from(err: io::Error) -> SerializeError {
        SerializeError::Io(err)
    }
}

impl From<FromUtf8Error> for SerializeError {
    fn from(err: FromUtf8Error) -> SerializeError {
        SerializeError::Utf8(err)
    }
}

impl From<ParseError> for SerializeError {
    fn from(err: ParseError) -> SerializeError {
        SerializeError::Chrono(err)
    }
}

