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

