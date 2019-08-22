use std::io;

#[derive(Fail, Debug)]
pub enum RiemannClientError {
    #[fail(display = "IO error: {}", _0)]
    IoError(#[fail(cause)] io::Error),
}

impl From<io::Error> for RiemannClientError {
    fn from(e: io::Error) -> RiemannClientError {
        RiemannClientError::IoError(e)
    }
}
