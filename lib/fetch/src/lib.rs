mod pool;

use {
    pool::ThreadPool,
    std::sync::mpsc,
    bc_track::TrackStream,
    bandcamp_api::{
        Api,
        Result,
        data::{
            Query,
            fans::Fan,
            search::Search,
            outlets::Outlet,
            releases::Release
        }
    }
};

pub enum Response {
    Fan(Result<Fan>),
    Search(Result<Search>),
    Outlet(Result<Outlet>),
    Release(Result<Release>),
    Track(Result<Box<bc_track::TrackStream>>)
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

from!(Fan, Result<Fan>);
from!(Search, Result<Search>);
from!(Outlet, Result<Outlet>);
from!(Release, Result<Release>);

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

    pub fn query<T, A>(&self, args: &A)
    where 
        T: Query<A> + Send + 'static,
        Result<T>: Into<Response>,
        A: ?Sized,
    {
        let api = self.api.clone();
        let sender = self.sender.clone();

        let req = api.request(args);

        self.pool.spawn(move || {
            let response = api.execute(req).into();
            let _ = sender.send(response);
        })
    }

    pub fn fetch_track(&self, url: reqwest::Url) {
        let api = self.api.clone();
        let sender = self.sender.clone();

        self.pool.spawn(move || {
            let stream = TrackStream::new(url, api.client().clone());
            let _ = sender.send(Response::Track(stream.map(Box::new)));
        })
    }
}
