//! Erros for serializing and deserializing

use std::{error, fmt::Display, str::Utf8Error};

///Errors for resend
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    ///value is zero for NonzeroInt
    Zero,
    ///invalid tag value for enum
    InvalidTag(u32),

    InvalidAscii(String),
    ///length is too big
    DataTooLarge(usize),

    InvalidChar(u32),

    Io(std::io::Error),

    Utf8(Utf8Error),
    //general error number
    ErroNo(u32),

    ///New kind will be added to replace Other
    Other(&'static str),
}

use Error::*;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Io(e) => Some(e),
            Utf8(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Io(e)
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Utf8(e)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Other(s)
    }
}
