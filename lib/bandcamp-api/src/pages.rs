use {
    select::document::Document
};

pub trait Page<A: ?Sized>
where
    Self: From<Document> + std::ops::Deref<Target = Document>,
    for <'url> &'url <Self as Page<A>>::Url: reqwest::IntoUrl
{
    type Url;

    fn url(args: &A) -> Self::Url;
}

macro_rules! impls {
    ($page:path) => {
        impl From<Document> for $page {
            fn from(doc: Document) -> Self {
                $page(doc)
            }
        }

        impl std::ops::Deref for $page {
            type Target = Document;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

macro_rules! url {
    ($($sub:literal .)? bandcamp.com / $path:literal, $($arg:expr),*) => {
        format!(concat!("https://", $($sub, ".",)? "bandcamp.com/", $path), $($arg),*)
    };
}

#[derive(Debug)]
pub struct Album(Document);

impl <'a> Page<AlbumArgs<'a>> for Album {
    type Url = String;

    fn url(args: &AlbumArgs) -> Self::Url {
        url!("{}".bandcamp.com/"album/{}", args.artist, args.name)
    }
}

impls!(Album);

pub struct AlbumArgs<'a> {
    pub artist: &'a str,
    pub name: &'a str
}

#[derive(Debug)]
pub struct Search(Document);

impl <'a> Page<SearchArgs<'a>> for Search {
    type Url = String;

    fn url(args: &SearchArgs) -> Self::Url {
        url!(bandcamp.com/"search?q={}&page={}", args.query, args.page)
    }
}

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

impls!(Search);
