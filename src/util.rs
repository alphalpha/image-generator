use std::{error, fmt, io, num};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ParseFloat(num::ParseFloatError),
    ParseInt(num::ParseIntError),
    Custom(String),
    Else,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::ParseFloat(ref err) => write!(f, "Parse Error: {}", err),
            Error::ParseInt(ref err) => write!(f, "Parse Error: {}", err),
            Error::Custom(ref err) => write!(f, "Error: {}", err),
            Error::Else => write!(f, "Some Error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::ParseFloat(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::Custom(ref err) => err,
            Error::Else => "Some Error",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::ParseFloat(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            Error::Custom(_) => None,
            Error::Else => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(err: num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Custom(err)
    }
}
