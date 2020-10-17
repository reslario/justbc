pub mod data;
pub mod pages;

use {
    pages::Page,
    scrape::Scraper,
    std::fmt::Display,
    reqwest::blocking::Client,
    snafu::{Snafu, ResultExt}
};

#[derive(Debug, Snafu)]
pub enum QueryError<DE>
where DE: std::fmt::Display + snafu::Error + 'static {
    #[snafu(display("connection error: {}", source))]
    Connection { source: reqwest::Error },
    #[snafu(display("error retrieving data: {}", source))]
    Data { source: DE }
}

pub type QueryResult<T, DE> = Result<T, QueryError<DE>>;

pub struct Api {
    client: Client
}

impl Api {
    pub fn new() -> Api {
        Api::with_client(Client::new())
    }

    pub fn with_client(client: Client) -> Api {
        Api { client }
    }

    pub fn query<T, P, A>(&self, args: &A) -> QueryResult<T, T::Err>
    where 
        T: data::Query<P>,
        P: Page<A>,
        A: ?Sized,
        T::Err: snafu::Error + Display + 'static,
        for <'url> &'url <P as Page<A>>::Url: reqwest::IntoUrl
    {
        let resp = self
            .client
            .get(&P::url(args))
            .send()
            .context(Connection)?
            .text()
            .context(Connection)
            .map(std::io::Cursor::new)?;

        let page = Scraper::new(resp).into();

        T::query(page).context(Data)
    }
}
