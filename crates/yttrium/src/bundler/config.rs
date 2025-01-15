use reqwest::Url;

pub struct BundlerConfig {
    url: Url,
}

impl BundlerConfig {
    pub fn new(url: Url) -> Self {
        BundlerConfig { url }
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }
}
