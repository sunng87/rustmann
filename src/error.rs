use std::io;

use thiserror::Error;

/// The error type
#[derive(Error, Debug)]
pub enum RiemannClientError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Riemann error: {0}")]
    RiemannError(String),
}
