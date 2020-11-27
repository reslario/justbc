type Scraper = scrape::Scraper<std::io::Cursor<bytes::Bytes>>;

pub trait Page<A: ?Sized>
where
    Self: From<Scraper> + std::ops::DerefMut<Target = Scraper>,
    for <'url> &'url <Self as Page<A>>::Url: reqwest::IntoUrl
{
    type Url;

    fn url(args: &A) -> Self::Url;
}

impl <P> Page<reqwest::Url> for P
where P: From<Scraper> + std::ops::DerefMut<Target = Scraper> {
    type Url = String;

    #[inline]
    fn url(url: &reqwest::Url) -> Self::Url {
        url.clone().into_string()
    }
} 

macro_rules! impls {
    ($page:path) => {
        impl From<Scraper> for $page {
            fn from(scraper: Scraper) -> Self {
                $page(scraper)
            }
        }

        impl std::ops::Deref for $page {
            type Target = Scraper;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $page {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

macro_rules! url {
    ($($sub:literal .)? bandcamp.com $(/ $path:literal)?, $($arg:expr),*) => {
        format!(concat!("https://", $($sub, ".",)? "bandcamp.com", $("/", $path)?), $($arg),*)
    };
}

pub struct Release(Scraper);

impl <'a> Page<ReleaseArgs<'a>> for Release {
    type Url = String;

    fn url(args: &ReleaseArgs) -> Self::Url {
        url!(
            "{}".bandcamp.com/"{}/{}",
            args.artist,
            args.kind.url_segment(),
            args.name
        )
    }
}

impls!(Release);

pub struct ReleaseArgs<'a> {
    pub kind: crate::data::releases::ReleaseKind,
    pub artist: &'a str,
    pub name: &'a str
}

pub struct Search(Scraper);

impl <'a> Page<SearchArgs<'a>> for Search {
    type Url = String;

    fn url(args: &SearchArgs) -> Self::Url {
        url!(bandcamp.com/"search?q={}&page={}", args.query, args.page)
    }
}

impls!(Search);

pub struct SearchArgs<'a> {
    query: &'a str,
    page: std::num::NonZeroU8
}

impl <'a> SearchArgs<'a> {
    pub fn query(query: &'a str) -> SearchArgs {
        SearchArgs {
            query,
            page: std::num::NonZeroU8::new(1).unwrap()
        }
    }
}

pub struct Outlet(Scraper);

impl Page<str> for Outlet {
    type Url = String;

    fn url(name: &str) -> Self::Url {
        url!("{}".bandcamp.com/"music", name)
    }
}

impls!(Outlet);
