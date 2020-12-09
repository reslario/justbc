mod pool;

use {
    pool::ThreadPool,
    std::sync::mpsc,
    bc_track::TrackStream,
    bandcamp_api::{
        Api,
        pages,
        QueryResult,
        data::{
            self,
            Query,
            search::Search,
            outlets::Outlet,
            releases::Release
        }
    }
};

type SearchResult = QueryResult<Search, <Search as Query<pages::Search>>::Err>;
type OutletResult = QueryResult<Outlet, <Outlet as Query<pages::Outlet>>::Err>;
type ReleaseResult = QueryResult<Release, <Release as Query<pages::Release>>::Err>;
type TrackResult = reqwest::Result<bc_track::TrackStream>;

pub enum Response {
    Search(SearchResult),
    Outlet(OutletResult),
    Release(ReleaseResult),
    Track(TrackResult)
}

macro_rules! from {
    ($variant:ident, $t:ty) => {
        impl From<$t> for Response {
            fn from(result: $t) -> Self {
                Response::$variant(result)
            }
        }
    };
}

from!(Search, SearchResult);
from!(Outlet, OutletResult);
from!(Release, ReleaseResult);

pub struct Fetcher {
    api: Api,
    pool: ThreadPool,
    sender: mpsc::Sender<Response>,
}

impl Fetcher {
    pub fn new(api: Api) -> (Fetcher, mpsc::Receiver<Response>) {
        let (sender, receiver) = mpsc::channel();

        let fetcher = Fetcher {
            api,
            pool: ThreadPool::new(),
            sender
        };

        (fetcher, receiver)
    }

    pub fn query<T, P, A>(&self, args: &A)
    where 
        T: data::Query<P> + Send + 'static,
        P: pages::Page<A>,
        A: ?Sized,
        T::Err: snafu::Error + std::fmt::Display + 'static,
        for <'url> &'url <P as pages::Page<A>>::Url: reqwest::IntoUrl,
        QueryResult<T, T::Err>: Into<Response>
    {
        let api = self.api.clone();
        let sender = self.sender.clone();

        let req = api.request(args);

        self.pool.spawn(move || {
            let response = api.execute(req).into();
            sender.send(response).unwrap()
        })
    }

    pub fn fetch_track(&self, url: reqwest::Url) {
        let api = self.api.clone();
        let sender = self.sender.clone();

        self.pool.spawn(move || {
            let stream = TrackStream::new(url, api.client().clone());
            sender.send(Response::Track(stream)).unwrap()
        })
    }
}
