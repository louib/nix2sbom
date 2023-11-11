use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("the data for key `{0}` is not available")]
    InvalidFormat(String),

    #[error("{0} is not supported yet.")]
    UnsupportedFormat(String),

    #[error("{0}")]
    UnknownError(String),
}
