use std::fs::File;
use std::io;
use std::io::{Write, Read};
use std::marker::Sized;
use std::string::FromUtf8Error;

pub trait Serialize where Self: Sized {
    fn serialize(&self, output: &mut Write) -> Result<(), SerializeError>;

    fn deserialize(input: &mut Read) -> Result<Self, SerializeError>;
}

#[derive(Debug)]
pub enum SerializeError {
    Io(io::Error),
    Utf8(FromUtf8Error),
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

pub trait PackedSize {
    fn packed_size() -> usize;
}
