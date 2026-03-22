use std::path::Path;

use crate::error::{Error, Result};

pub struct AppImage {
    #[allow(dead_code)]
    path: Box<Path>,
}

impl AppImage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::AppImage(format!(
                "File does not exist: {}",
                path.display()
            )));
        }

        if !path.is_file() {
            return Err(Error::AppImage(format!("Not a file: {}", path.display())));
        }

        Ok(Self { path: path.into() })
    }

    pub fn read_update_info(&self) -> Result<String> {
        todo!("Implement update info extraction from AppImage")
    }
}
