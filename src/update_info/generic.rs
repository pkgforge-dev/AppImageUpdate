#[derive(Debug, Clone)]
pub struct GenericUpdateInfo {
    pub url: String,
}

impl GenericUpdateInfo {
    pub fn zsync_url(&self) -> &str {
        &self.url
    }
}
