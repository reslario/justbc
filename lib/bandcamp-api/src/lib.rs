pub mod data;

#[cfg(feature = "query")]
mod url;

#[cfg(feature = "query")]
use {
    std::marker::PhantomData,
    reqwest::blocking::Client
};

#[cfg(feature = "query")]
pub struct Request<T> {
    inner: reqwest::blocking::Request,
    _marker: PhantomData<T>
}

#[cfg(feature = "query")]
pub type Result<T> = reqwest::Result<T>;

#[cfg(feature = "query")]
#[derive(Clone, Default)]
pub struct Api {
    client: Client
}

#[cfg(feature = "query")]
impl Api {
    pub fn new() -> Api {
        <_>::default()
    }

    pub fn with_client(client: Client) -> Api {
        Api { client }
    }

    pub fn query<T, A>(&self, args: &A) -> Result<T>
    where 
        T: data::Query<A>,
        A: ?Sized    
    {
        self.execute(self.request(args))
    }

    pub fn request<T, A>(&self, args: &A) -> Request<T>
    where 
        T: data::Query<A>,
        A: ?Sized    
    {
        let inner = self
            .client
            .get(T::url(args))
            .build()
            .unwrap();
        
        Request {
            inner,
            _marker: PhantomData
        }
    }

    pub fn execute<T, A>(&self, request: Request<T>) -> Result<T>
    where 
        T: data::Query<A>,
        A: ?Sized    
    {
        self.client
            .execute(request.inner)?
            .json()
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
