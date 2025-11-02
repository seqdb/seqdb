use std::{
    fmt::{self, Debug, Display},
    fs, io, result,
};

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    TryLock(fs::TryLockError),

    Str(&'static str),
    String(String),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<fs::TryLockError> for Error {
    fn from(value: fs::TryLockError) -> Self {
        Self::TryLock(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(error) => Display::fmt(&error, f),
            Error::TryLock(_) => write!(
                f,
                "Couldn't lock file. It must be already opened by another process."
            ),

            Error::Str(s) => write!(f, "{s}"),
            Error::String(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}
