pub mod data;

#[cfg(feature = "query")]
pub mod pages;

#[cfg(feature = "query")]
use {
    std::{
        fmt::Display,
        marker::PhantomData,
    },
    pages::Page,
    scrape::Scraper,
    reqwest::blocking::Client,
    snafu::{Snafu, ResultExt}
};

#[cfg(feature = "query")]
#[derive(Debug, Snafu)]
pub enum QueryError<DE>
where DE: std::fmt::Display + snafu::Error + 'static {
    #[snafu(display("connection error: {}", source))]
    Connection { source: reqwest::Error },
    #[snafu(display("error retrieving data: {}", source))]
    Data { source: DE }
}

#[cfg(feature = "query")]
pub type QueryResult<T, DE> = Result<T, QueryError<DE>>;

#[cfg(feature = "query")]
pub struct Request<T> {
    inner: reqwest::blocking::Request,
    _marker: PhantomData<T>
}

#[cfg(feature = "query")]
#[derive(Clone)]
pub struct Api {
    client: Client
}

#[cfg(feature = "query")]
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
        self.execute(self.request(args))
    }

    pub fn request<T, P, A>(&self, args: &A) -> Request<T>
    where 
        T: data::Query<P>,
        P: Page<A>,
        A: ?Sized,
        T::Err: snafu::Error + Display + 'static,
        for <'url> &'url <P as Page<A>>::Url: reqwest::IntoUrl
    {
        let inner = self
            .client
            .get(&P::url(args))
            .build()
            .unwrap();
        
        Request {
            inner,
            _marker: PhantomData
        }
    }

    pub fn execute<T, P, A>(&self, request: Request<T>) -> QueryResult<T, T::Err>
    where 
        T: data::Query<P>,
        P: Page<A>,
        A: ?Sized,
        T::Err: snafu::Error + Display + 'static,
        for <'url> &'url <P as Page<A>>::Url: reqwest::IntoUrl
    {
        self.client
            .execute(request.inner)
            .context(Connection)?
            .bytes()
            .context(Connection)
            .map(std::io::Cursor::new)
            .map(Scraper::new)
            .map(P::from)
            .map(T::query)?
            .context(Data)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
