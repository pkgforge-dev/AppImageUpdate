use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zsync_rs::{ControlFile, ZsyncAssembly};

use crate::appimage::AppImage;
use crate::error::{Error, Result};
use crate::update_info::UpdateInfo;

struct UpdateContext {
    source_size: u64,
    target_size: u64,
    block_size: usize,
    original_perms: Option<fs::Permissions>,
}

#[derive(Debug)]
pub struct UpdateStats {
    pub source_path: PathBuf,
    pub source_size: u64,
    pub target_path: PathBuf,
    pub target_size: u64,
    pub blocks_reused: usize,
    pub blocks_downloaded: usize,
    pub block_size: usize,
    pub backup_path: Option<PathBuf>,
}

impl UpdateStats {
    pub fn bytes_reused(&self) -> u64 {
        (self.blocks_reused * self.block_size) as u64
    }

    pub fn bytes_downloaded(&self) -> u64 {
        (self.blocks_downloaded * self.block_size) as u64
    }

    pub fn saved_percentage(&self) -> u64 {
        if self.target_size == 0 {
            return 0;
        }
        (self.bytes_reused() * 100 / self.target_size).min(100)
    }
}

pub struct Updater {
    appimage: AppImage,
    update_info: UpdateInfo,
    output_dir: PathBuf,
    overwrite: bool,
    progress_callback: Option<Arc<dyn Fn(u64, u64) + Send + Sync>>,
}

impl Updater {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let appimage = AppImage::open(path)?;
        let update_info_str = appimage.read_update_info()?;
        let update_info = UpdateInfo::parse(&update_info_str)?;

        let output_dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();

        Ok(Self {
            appimage,
            update_info,
            output_dir,
            overwrite: false,
            progress_callback: None,
        })
    }

    pub fn with_update_info<P: AsRef<Path>>(path: P, update_info: &str) -> Result<Self> {
        let path = path.as_ref();
        let appimage = AppImage::open(path)?;
        let update_info = UpdateInfo::parse(update_info)?;

        let output_dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();

        Ok(Self {
            appimage,
            update_info,
            output_dir,
            overwrite: false,
            progress_callback: None,
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

    pub fn progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(callback));
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
        if self.overwrite {
            return Ok(self.appimage.path().to_path_buf());
        }
        let filename = control
            .filename
            .as_ref()
            .ok_or_else(|| Error::Zsync("Control file has no filename".into()))?;
        Ok(self.output_dir.join(filename))
    }

    fn verify_existing_file(&self, path: &Path, expected_sha1: &str) -> Result<bool> {
        use sha1::{Digest, Sha1};

        let mut file = fs::File::open(path)?;
        let mut hasher = Sha1::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        let actual_sha1 = hex::encode(hash);

        Ok(actual_sha1.eq_ignore_ascii_case(expected_sha1))
    }

    pub fn check_for_update(&self) -> Result<bool> {
        let (control, _zsync_url) = self.fetch_control_file()?;

        if let Some(ref expected_sha1) = control.sha1
            && self.verify_existing_file(self.appimage.path(), expected_sha1)?
        {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn source_path(&self) -> &Path {
        self.appimage.path()
    }

    fn source_size(&self) -> u64 {
        fs::metadata(self.appimage.path())
            .map(|m| m.len())
            .unwrap_or(0)
    }

    pub fn update_info(&self) -> &str {
        self.update_info.raw()
    }

    pub fn zsync_url(&self) -> Result<String> {
        self.update_info.zsync_url()
    }

    pub fn target_info(&self) -> Result<(PathBuf, u64)> {
        let (control, _zsync_url) = self.fetch_control_file()?;
        let output_path = self.resolve_output_path(&control)?;
        Ok((output_path, control.length))
    }

    pub fn perform_update(&self) -> Result<(PathBuf, UpdateStats)> {
        let (control, zsync_url) = self.fetch_control_file()?;
        let output_path = self.resolve_output_path(&control)?;

        if output_path.exists() {
            if let Some(ref expected_sha1) = control.sha1
                && self.verify_existing_file(&output_path, expected_sha1)?
            {
                let stats = UpdateStats {
                    source_path: self.appimage.path().to_path_buf(),
                    source_size: self.source_size(),
                    target_path: output_path.clone(),
                    target_size: control.length,
                    blocks_reused: 0,
                    blocks_downloaded: 0,
                    block_size: control.blocksize,
                    backup_path: None,
                };
                return Ok((output_path, stats));
            }

            let same_file = self.appimage.path() == output_path;
            if !same_file && !self.overwrite {
                return Err(Error::AppImage(format!(
                    "Output file already exists: {}",
                    output_path.display()
                )));
            }
        }

        let ctx = UpdateContext {
            source_size: self.source_size(),
            target_size: control.length,
            block_size: control.blocksize,
            original_perms: fs::metadata(self.appimage.path())
                .map(|m| m.permissions())
                .ok(),
        };

        let source_path = self.appimage.path();
        let same_file = source_path == output_path;

        let (actual_source_path, backup_path) = if same_file {
            let filename = source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("appimage");
            let backup = source_path.with_file_name(format!("{}.old", filename));
            let _ = fs::remove_file(&backup);
            fs::rename(source_path, &backup)?;
            (backup.clone(), Some(backup))
        } else {
            (source_path.to_path_buf(), None)
        };

        let result = self.do_update(&actual_source_path, &output_path, &zsync_url, &ctx);

        match result {
            Ok(mut stats) => {
                stats.backup_path = backup_path;
                Ok((output_path, stats))
            }
            Err(e) => {
                if let Some(backup) = backup_path {
                    let _ = fs::rename(&backup, source_path);
                }
                Err(e)
            }
        }
    }

    fn do_update(
        &self,
        source_path: &Path,
        output_path: &Path,
        zsync_url: &str,
        ctx: &UpdateContext,
    ) -> Result<UpdateStats> {
        let mut assembly = ZsyncAssembly::from_url(zsync_url, output_path)
            .map_err(|e| Error::Zsync(format!("Failed to initialize zsync: {}", e)))?;

        if let Some(ref callback) = self.progress_callback {
            let callback = callback.clone();
            assembly.set_progress_callback(move |done, total| callback(done, total));
        }

        let blocks_reused = assembly
            .submit_source_file(source_path)
            .map_err(|e| Error::Zsync(format!("Failed to submit source file: {}", e)))?;

        let self_blocks = assembly
            .submit_self_referential()
            .map_err(|e| Error::Zsync(format!("Self-referential scan failed: {}", e)))?;
        let blocks_reused = blocks_reused.saturating_add(self_blocks);

        let blocks_downloaded = assembly
            .download_missing_blocks()
            .map_err(|e| Error::Zsync(format!("Failed to download blocks: {}", e)))?;

        assembly
            .complete()
            .map_err(|e| Error::Zsync(format!("Failed to complete assembly: {}", e)))?;

        if let Some(ref perms) = ctx.original_perms {
            fs::set_permissions(output_path, perms.clone())?;
        }

        Ok(UpdateStats {
            source_path: self.appimage.path().to_path_buf(),
            source_size: ctx.source_size,
            target_path: output_path.to_path_buf(),
            target_size: ctx.target_size,
            blocks_reused,
            blocks_downloaded,
            block_size: ctx.block_size,
            backup_path: None,
        })
    }
}
