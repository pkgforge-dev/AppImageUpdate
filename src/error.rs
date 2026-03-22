use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid update information: {0}")]
    InvalidUpdateInfo(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("AppImage error: {0}")]
    AppImage(String),
}

pub type Result<T> = std::result::Result<T, Error>;
