pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("ROM file is too big: {0} bytes expected < 3583 bytes.")]
    ROMIsTooBig(u64),
}
