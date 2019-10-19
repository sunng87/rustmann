use std::io;

/// The error type
#[derive(Fail, Debug)]
pub enum RiemannClientError {
    #[fail(display = "IO error: {}", _0)]
    IoError(#[fail(cause)] io::Error),
    #[fail(display = "Riemann error: {}", _0)]
    RiemannError(String),
}

impl From<io::Error> for RiemannClientError {
    fn from(e: io::Error) -> RiemannClientError {
        RiemannClientError::IoError(e)
    }
}
