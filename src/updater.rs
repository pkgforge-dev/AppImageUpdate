use std::path::{Path, PathBuf};

use crate::appimage::AppImage;
use crate::error::{Error, Result};
use crate::update_info::UpdateInfo;

pub struct Updater {
    #[allow(dead_code)]
    appimage: AppImage,
    #[allow(dead_code)]
    update_info: UpdateInfo,
    output_dir: PathBuf,
    overwrite: bool,
}

impl Updater {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let appimage = AppImage::open(path)?;
        let update_info_str = appimage.read_update_info()?;
        let update_info = UpdateInfo::parse(&update_info_str)?;

        Ok(Self {
            appimage,
            update_info,
            output_dir: std::env::current_dir().map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "Failed to get current directory: {}",
                    e
                )))
            })?,
            overwrite: false,
        })
    }

    pub fn with_update_info<P: AsRef<Path>>(path: P, update_info: &str) -> Result<Self> {
        let appimage = AppImage::open(path.as_ref())?;
        let update_info = UpdateInfo::parse(update_info)?;

        Ok(Self {
            appimage,
            update_info,
            output_dir: std::env::current_dir().map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "Failed to get current directory: {}",
                    e
                )))
            })?,
            overwrite: false,
        })
    }

    pub fn output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    pub fn check_for_update(&self) -> Result<bool> {
        todo!("Implement update check")
    }

    pub fn perform_update(&self) -> Result<PathBuf> {
        todo!("Implement update")
    }
}
