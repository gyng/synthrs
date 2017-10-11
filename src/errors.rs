use std;
use std::error;
use std::fmt;
use std::io;

#[allow(dead_code)]
/// Re-exported `Result` for rcue errors
pub type Result<T> = std::result::Result<T, SynthrsError>;

#[allow(dead_code)]
#[derive(Debug)]
/// Represents a parsing error.
pub enum SynthrsError {
    /// File/format parse error
    Parse(String),
    /// IO error (file could not read)
    Io(io::Error),
}

impl fmt::Display for SynthrsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SynthrsError::Parse(ref token) => write!(f, "Parse error: {}", token),
            SynthrsError::Io(ref err) => write!(f, "Io error: {}", err),
        }
    }
}

impl error::Error for SynthrsError {
    fn description(&self) -> &str {
        match *self {
            SynthrsError::Parse(ref token) => token,
            SynthrsError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            SynthrsError::Parse(ref _token) => None,
            SynthrsError::Io(ref err) => err.cause(),
        }
    }
}

impl From<io::Error> for SynthrsError {
    fn from(err: io::Error) -> Self {
        SynthrsError::Io(err)
    }
}
