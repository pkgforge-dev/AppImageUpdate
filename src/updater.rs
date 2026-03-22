use std::path::{Path, PathBuf};

use zsync_rs::{ControlFile, ZsyncAssembly};

use crate::appimage::AppImage;
use crate::error::{Error, Result};
use crate::update_info::UpdateInfo;

pub struct Updater {
    appimage: AppImage,
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

    fn fetch_control_file(&self) -> Result<(ControlFile, String)> {
        let zsync_url = self.update_info.zsync_url()?;
        let http = zsync_rs::HttpClient::new();
        let control = http
            .fetch_control_file(&zsync_url)
            .map_err(|e| Error::Zsync(format!("Failed to fetch control file: {}", e)))?;
        Ok((control, zsync_url))
    }

    fn resolve_output_path(&self, control: &ControlFile) -> Result<PathBuf> {
        let filename = control
            .filename
            .as_ref()
            .ok_or_else(|| Error::Zsync("Control file has no filename".into()))?;
        Ok(self.output_dir.join(filename))
    }

    pub fn check_for_update(&self) -> Result<bool> {
        let (control, _zsync_url) = self.fetch_control_file()?;

        let output_path = self.resolve_output_path(&control)?;
        if output_path.exists() && !self.overwrite {
            return Err(Error::AppImage(format!(
                "Output file already exists: {}",
                output_path.display()
            )));
        }

        Ok(true)
    }

    pub fn perform_update(&self) -> Result<PathBuf> {
        let (control, zsync_url) = self.fetch_control_file()?;
        let output_path = self.resolve_output_path(&control)?;

        if output_path.exists() && !self.overwrite {
            return Err(Error::AppImage(format!(
                "Output file already exists: {}",
                output_path.display()
            )));
        }

        let assembly = ZsyncAssembly::from_url(&zsync_url, &output_path)
            .map_err(|e| Error::Zsync(format!("Failed to initialize zsync: {}", e)))?;

        let mut assembly = assembly;

        assembly
            .submit_source_file(self.appimage.path())
            .map_err(|e| Error::Zsync(format!("Failed to submit source file: {}", e)))?;

        assembly
            .submit_self_referential()
            .map_err(|e| Error::Zsync(format!("Self-referential scan failed: {}", e)))?;

        assembly
            .download_missing_blocks()
            .map_err(|e| Error::Zsync(format!("Failed to download blocks: {}", e)))?;

        assembly
            .complete()
            .map_err(|e| Error::Zsync(format!("Failed to complete assembly: {}", e)))?;

        Ok(output_path)
    }

    pub fn progress(&self) -> Option<(u64, u64)> {
        None
    }
}
