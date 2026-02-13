use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrekError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No repository found in current directory")]
    NoRepository,

    #[error("No commits found in repository")]
    NoCommits,

    #[error("Invalid UTF-8 in file content")]
    InvalidUtf8,
}

pub type Result<T> = std::result::Result<T, TrekError>;
