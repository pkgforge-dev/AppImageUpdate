pub mod appimage;
pub mod config;
pub mod error;
pub mod update_info;
pub mod updater;

pub use error::Error;
pub use updater::{UpdateStats, Updater};
