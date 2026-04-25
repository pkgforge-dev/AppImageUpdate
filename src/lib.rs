pub mod appimage;
pub mod config;
pub mod error;
pub mod self_update;
pub mod update_info;
pub mod updater;
pub mod util;

pub use appimage::AppImageType;
pub use error::Error;
pub use update_info::{ForgeKind, ForgeUpdateInfo, GenericUpdateInfo, UpdateInfo};
pub use updater::{UpdateStats, Updater};
