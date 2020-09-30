use {
    std::io::BufRead,
    crate::{filter, Result},
    quick_xml::{
        events::*,
        Reader
    }
};

/// Type alias for a [Vec](std::vec::Vec) of bytes.
pub type Buf = Vec<u8>;

/// Type alias for a mutable reference to a [Buf](Buf).
pub type BufMut<'a> = &'a mut Buf;

/// Defines the interface for Scrapers.
pub trait Scrape: Sized {
    /// Reads the next [Event](crate::Event) in an xml document.
    fn read_event<'b>(&mut self, buf: &'b mut Buf) -> Result<Event<'b>>;

    /// Steps over the next [Event](crate::Event).
    ///
    /// Clears the provided buffer afterwards.
    fn step(&mut self, buf: BufMut) -> Result<&mut Self> {
        self.read_event(buf)?;
        buf.clear();
        Ok(self)
    }

    /// Reads the next [Event](crate::Event) and extracts something from it
    /// using the provided function.
    /// See the [extract](crate::extract) module for some built-in extraction
    /// functions.
    ///
    /// Clears the provided buffer afterwards.
    fn extract<F, T>(&mut self, mut f: F, buf: BufMut) -> Result<Option<T>>
    where F: for<'evt> FnMut(Event<'evt>) -> Option<Result<T>> {
        let event = self.read_event(buf)?;
        let res = f(event).transpose();
        buf.clear();
        res
    }


    /// Restricts a scraper to a certain depth in the xml tree.
    /// E.g. filtering by the tag `div` will only yield events related
    /// to that tag and its descendants.
    ///
    /// See the [filter](crate::filter) module for some built-in filters.
    fn filter<'a, F>(&'a mut self, filter: F) -> Filtered<&'a mut Self, F>
    where F: for<'evt> filter::Filter<'evt> {
        Filtered::new(self, filter)
    }

    /// Like the [filter](Scrape::filter) method, but takes ownership
    /// of the scraper.
    fn into_filter<'a, F>(self, filter: F) -> Filtered<Self, F>
    where F: for<'evt> filter::Filter<'evt> {
        Filtered::new(self, filter)
    }

    /// Runs the provided extraction function until it returns `Some` or
    /// [Eof](crate::Event::Eof) is reached.
    fn find_extract<F, T>(&mut self, mut f: F, buf: BufMut) -> Result<Option<T>>
    where F: for<'evt> FnMut(Event<'evt>) -> Option<Result<T>> {
        loop {
            match self.read_event(buf)? {
                Event::Eof => {
                    buf.clear();
                    return Ok(None)
                },
                event => if let Some(result) = f(event) {
                    buf.clear();
                    return Some(result).transpose()
                } else {
                    buf.clear()
                }
            }
        }
    }
}

impl <S: Scrape> Scrape for &mut S {
    fn read_event<'b>(&mut self, buf: BufMut<'b>) -> Result<Event<'b>> {
        (*self).read_event(buf)
    }
}

/// Provides a straight-forward implementation of the [Scrape](Scrape)
/// trait. It is usually the initial type you'll construct to start
/// scraping from a reader.
pub struct Scraper<R>
where R: BufRead {
    reader: Reader<R>
}

impl <R> Scraper<R>
where R: BufRead {
    /// Creates a new `Scraper` from a reader.
    pub fn new(reader: R) -> Scraper<R> {
        Scraper {
            reader: Self::configure_reader(Reader::from_reader(reader)),
        }
    }

    fn configure_reader(mut reader: Reader<R>) -> Reader<R> {
        reader.trim_text(true)
            .check_end_names(false);
        reader
    }
}

impl <R> Scrape for Scraper<R>
where R: BufRead {
    fn read_event<'b>(&mut self, buf: BufMut<'b>) -> Result<Event<'b>> {
        self.reader.read_event(buf)
    }
}

/// A filtered scraper created using [Scrape::filter](Scrape::filter).
pub struct Filtered<S, F> {
    scraper: S,
    filter: F,
    inside: bool,
}

impl <'src, S, F> Filtered<S, F>
where 
    F: for<'evt> filter::Filter<'evt>,
    S: Scrape
{
    fn new(scraper: S, filter: F) -> Self {
        Filtered {
            scraper,
            filter,
            inside: false
        }
    }

    fn next<'b>(&mut self, buf: BufMut<'b>) -> Result<Event<'b>> {
        loop {
            let event = self.scraper.read_event(reborrow(buf))?;
            match event {
                Event::Eof => return Ok(event),
                event if self.filter.start(&event) => {
                    self.inside = true;
                    return Ok(event)
                },
                _ => {
                    reborrow(buf).clear()
                }
            }
        }

        // this is necessary so we can repeatedly access
        // the buffer from within the loop
        //
        // since we don't hand out any of these borrows
        // to outside the function, this should be fine
        fn reborrow<'a>(buf: *mut Buf) -> BufMut<'a> {
            unsafe { &mut *buf }
        }
    }

    /// Creates a scraper that only reads `n` instances
    /// of the set of events that this filtered scraper
    /// produces and will emit [Eof](crate::Event::Eof) after that.
    pub fn take(&mut self, n: usize) -> Take<S, F, &mut Self> {
        Take::new(self, n)
    } 

    pub fn into_take(self, n: usize) -> Take<S, F, Self> {
        Take::new(self, n)
    }
}

impl <'src, S, F> Scrape for Filtered<S, F>
where 
    F: for<'evt> filter::Filter<'evt>,
    S: Scrape
{
    fn read_event<'b>(&mut self, buf: &'b mut Buf) -> Result<Event<'b>> {
        if self.inside {
            let event = self.scraper.read_event(buf)?;
            self.inside = !self.filter.end(&event);
            Ok(event)
        } else {
            self.next(buf)
        }
    }
}

/// Scraper created by [Filtered::take](Filtered::take).
pub struct Take<S, F, B> {
    filtered: B,
    taken: usize,
    n: usize,
    _marker: std::marker::PhantomData<(S, F)>
}

impl <S, F, B> Take<S, F, B> {
    fn new(filtered: B, n: usize) -> Self {
        Take {
            filtered,
            taken: 0,
            n,
            _marker: <_>::default()
        }
    }
}

impl <S, F, B> Scrape for Take<S, F, B>
where 
    B: std::borrow::BorrowMut<Filtered<S, F>>,
    F: for<'evt> filter::Filter<'evt>,
    S: Scrape
{
    fn read_event<'b>(&mut self, buf: &'b mut Buf) -> Result<Event<'b>> {
        let filtered = self.filtered.borrow_mut();
        
        if filtered.inside {
            let event = filtered.scraper.read_event(buf)?;
            filtered.inside = !filtered.filter.end(&event);
            Ok(event)
        } else {
            if self.taken == self.n {
                Ok(Event::Eof)
            } else {
                self.taken += 1;
                filtered.next(buf)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        filter::*,
        crate::extract
    };

    const HTML: &str = r#"
        <stuff />
        <div class="outer">
            <div class="interesting">
                <div class="type">
                    TYPE    
                </div>
                <div class="heading">
                    <a href="some.link/">
                        &lt;content&gt;
                    </a>
                </div>
            </div>
        </div>
    "#;

    #[test]
    fn general_test() {
        let html = HTML.repeat(2);

        let mut scraper = Scraper::new(html.as_bytes());
        let mut interesting = scraper.filter(div().class("interesting"));
        
        scrape(&mut interesting).unwrap();
        scrape(interesting).unwrap()
    }

    fn scrape(mut scraper: impl Scrape) -> Result<()> {
        let mut buf = vec![];
        let ty = scraper
            .filter(div().class("type"))
            .step(&mut buf)?
            .extract(extract::text, &mut buf)?
            .unwrap();
    
        assert_eq!(ty, "TYPE");

        let mut a = scraper
            .filter(div().class("heading"))
            .into_filter(tag("a"));

        let href = a.extract(extract::attr("href"), &mut buf)?
            .unwrap();

        assert_eq!(href, "some.link/");

        let content = a.extract(extract::text, &mut buf)?
            .unwrap();

        assert_eq!(content, "<content>");

        Ok(())
    }

    #[test]
    fn take() {
        let html = HTML.repeat(2);

        let mut scraper = Scraper::new(html.as_bytes());
        let mut interesting = scraper.filter(div().class("interesting"));
        let mut interesting = interesting.take(1);

        scrape(&mut interesting).unwrap();
        
        match interesting
            .filter(div().class("type"))
            .read_event(&mut vec![]) 
        {
            Ok(Event::Eof) => { /* good */ },
            event => panic!("expected `Ok(Event::Eof)`, got {:?}", event)
        }
    }
}
