use std::{error::Error, fmt};

#[derive(Debug)]
pub(crate) struct CipherError {
    kind: ErrorKind,
    error: String,
}

impl CipherError {
    pub fn new(kind: ErrorKind, error: String) -> Self {
        Self { kind, error }
    }
}

impl Error for CipherError {}

impl fmt::Display for CipherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.error)
    }
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    KeyError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::KeyError => write!(f, "KeyError"),
        }
    }
}

pub(crate) type CipherResult<T> = Result<T, CipherError>;
