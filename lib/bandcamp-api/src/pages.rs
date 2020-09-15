use {
    select::document::Document
};

pub trait Page
where
    Self: From<Document> + std::ops::Deref<Target = Document>,
    for <'url> &'url <Self as Page>::Url: reqwest::IntoUrl
{
    type Args: ?Sized;
    type Url;

    fn url(args: &Self::Args) -> Self::Url;
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

impl Page for Album {
    type Args = AlbumArgs;
    type Url = String;

    fn url(args: &Self::Args) -> Self::Url {
        url!("{}".bandcamp.com/"album/{}", args.artist, args.name)
    }
}

impls!(Album);

pub struct AlbumArgs {
    pub artist: String,
    pub name: String
}

