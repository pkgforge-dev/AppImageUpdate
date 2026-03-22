use std::sync::OnceLock;

static GITHUB_API_PROXIES: OnceLock<Vec<String>> = OnceLock::new();

pub fn init(proxies: Option<String>) {
    let proxies = proxies
        .or_else(|| std::env::var("GITHUB_API_PROXY").ok())
        .map(|s| {
            s.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let _ = GITHUB_API_PROXIES.set(proxies);
}

pub fn get_proxies() -> &'static [String] {
    GITHUB_API_PROXIES
        .get()
        .map(|v| v.as_slice())
        .unwrap_or(&[])
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
