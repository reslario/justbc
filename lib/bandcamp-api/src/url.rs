use url::Url;

pub struct Base;
pub struct WithVersion;
pub struct WithFunction;

const BASE: &str = "https://bandcamp.com/api";

pub struct ApiUrl<S> {
    url: Url,
    _state: S
}

impl ApiUrl<Base> {
    pub fn new(path: impl AsRef<str>) -> Self {
        let mut url = BASE
            .parse::<Url>()
            .unwrap();

        url.path_segments_mut()
            .unwrap()
            .push(path.as_ref());

        ApiUrl { 
            url,
            _state: Base 
        }
    }

    pub fn mobile() -> ApiUrl<WithVersion> {
        ApiUrl::new("mobile")
            .version("24")
    }

    pub fn version(mut self, version: impl AsRef<str>) -> ApiUrl<WithVersion> {
        self.url
            .path_segments_mut()
            .unwrap()
            .push(version.as_ref());

        ApiUrl {
            url: self.url,
            _state: WithVersion
        }
    }
}

impl ApiUrl<WithVersion> {
    pub fn function(mut self, function: impl AsRef<str>) -> ApiUrl<WithFunction> {
        self.url
            .path_segments_mut()
            .unwrap()
            .push(function.as_ref())
            .push("1");

        ApiUrl {
            url: self.url,
            _state: WithFunction
        }
    }
}

impl ApiUrl<WithFunction> {
    pub fn query(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.url
            .query_pairs_mut()
            .append_pair(key.as_ref(), value.as_ref());

        self
    }
}

impl From<ApiUrl<WithFunction>> for Url {
    fn from(api_url: ApiUrl<WithFunction>) -> Self {
        api_url.url
    }
}
