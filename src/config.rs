use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

impl Default for StringOrVec {
    fn default() -> Self {
        StringOrVec::Vec(Vec::new())
    }
}

impl StringOrVec {
    fn to_vec(&self) -> Vec<String> {
        match self {
            StringOrVec::String(s) => parse_proxies(s),
            StringOrVec::Vec(v) => v.clone(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub github_api_proxy: Option<StringOrVec>,
    pub remove_old: Option<bool>,
    pub output_dir: Option<PathBuf>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
static PROXIES: OnceLock<Vec<String>> = OnceLock::new();

pub fn init() {
    let config = load_config();
    let _ = CONFIG.set(config);
}

pub fn set_proxies(proxies: Vec<String>) {
    if !proxies.is_empty() {
        let _ = PROXIES.set(proxies);
    }
}

pub fn get() -> &'static Config {
    static DEFAULT: Config = Config {
        github_api_proxy: None,
        remove_old: None,
        output_dir: None,
    };
    CONFIG.get().unwrap_or(&DEFAULT)
}

fn load_config() -> Config {
    let portable = std::env::current_exe().ok().and_then(|exe| {
        let dir = exe.parent()?;
        let name = exe.file_stem()?.to_str()?;
        Some(dir.join(format!("{name}.toml")))
    });

    let user = dirs::config_dir().map(|p| p.join("appimageupdate/config.toml"));

    let global = Some(PathBuf::from("/etc/appimageupdate/config.toml"));

    portable
        .into_iter()
        .chain(user)
        .chain(global)
        .find_map(try_load_config)
        .unwrap_or_default()
}

fn try_load_config(path: PathBuf) -> Option<Config> {
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
}

pub fn get_proxies() -> Vec<String> {
    if let Some(proxies) = PROXIES.get() {
        return proxies.clone();
    }

    if let Ok(s) = std::env::var("GITHUB_API_PROXY") {
        return parse_proxies(&s);
    }

    get()
        .github_api_proxy
        .as_ref()
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

pub fn get_remove_old(cli_value: Option<bool>) -> bool {
    if let Some(v) = cli_value {
        return v;
    }
    if let Ok(v) = std::env::var("APPIMAGEUPDATE_REMOVE_OLD")
        && let Ok(b) = v.parse::<bool>()
    {
        return b;
    }
    get().remove_old.unwrap_or(false)
}

pub fn get_output_dir(cli_value: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = cli_value {
        return Some(expand_path(&dir.to_string_lossy()));
    }
    if let Ok(dir) = std::env::var("APPIMAGEUPDATE_OUTPUT_DIR") {
        return Some(expand_path(&dir));
    }
    get()
        .output_dir
        .as_ref()
        .map(|p| expand_path(&p.to_string_lossy()))
}

fn expand_path(s: &str) -> PathBuf {
    PathBuf::from(shellexpand::full(s).unwrap_or_default().into_owned())
}

fn parse_proxies(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn build_api_url(path: &str, proxy: Option<&str>) -> String {
    match proxy {
        Some(proxy) => {
            let proxy = proxy.trim_end_matches('/');
            format!("{}{}", proxy, path)
        }
        None => format!("https://api.github.com{}", path),
    }
}
