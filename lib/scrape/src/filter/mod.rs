//! Filters for restricting a scraper's output.

mod pat;

use quick_xml::events::*;

pub use pat::BytePat;

/// Defines the interface for a filter, which consists
/// of two functions that match the start and end of
/// a set of [events](crate::Event).
pub trait Filter<'evt> {
    /// Determines whether an event marks the start of
    /// a group of events we're filtering for
    fn start(&mut self, event: &Event<'evt>) -> bool;
    /// Determines whether an event marks the end of
    /// a group of events we're filtering for
    fn end(&mut self, event: &Event<'evt>) -> bool;

    /// Limits the number of events permitted by this
    /// Filter to `n`. Useful if you want to filter for
    /// unclosed tags, like html's `<meta>`.
    fn take(self, n: usize) -> Take<Self>
    where Self: Sized {
        Take::new(self, n)
    }
}

/// Filter created by [Filter::take](Filter::take).
pub struct Take<F> {
    filter: F,
    taken: usize,
    n: usize
}

impl <F> Take<F> {
    fn new(filter: F, n: usize) -> Take<F> {
        Take {
            filter,
            taken: 0,
            n
        }
    }
}

impl <'evt, F> Filter<'evt> for Take<F> 
where F: Filter<'evt> {
    fn start(&mut self, event: &Event<'evt>) -> bool {
        self.filter.start(event)
    }

    fn end(&mut self, event: &Event<'evt>) -> bool {
        if self.taken == self.n {
            true
        } else {
            self.taken += 1;
            self.filter.end(event)
        }
    }
}

/// Filter created by the [tag](tag) function. 
#[derive(Clone, Copy)]
pub struct Tag<P> {
    name: P,
    opened: usize
}

/// Filters for events that match the provided tag name.
pub fn tag<P: BytePat>(name: P) -> Tag<P> {
    Tag { name, opened: 0 }
}

/// Filters events that match the `div` tag.
///
/// Shorthand for `.tag("div")`
pub fn div() -> Tag<&'static str> {
    tag("div")
}

macro_rules! shorthand {
    ($name:ident, $attr:expr) => {
        #[doc = "Filters for events that have the provided `"]
        #[doc = $attr]
        #[doc = "` attribute, in addition to the tag name.\n"]
        #[doc = "Shorthand for `.attr(\"`᠎`"]
        #[doc = $attr]
        #[doc = "`᠎`\", <value>)`"]
        pub fn $name<VP: BytePat>(self, value: VP) -> Attr<P, &'static str, VP> {
            self.attr($attr, value)
        }
    };

    ($name:ident) => {
        shorthand!($name, stringify!($name));
    }
}

impl <P: BytePat> Tag<P> {
    fn matches(&self, name: &[u8]) -> bool {
        self.name.matches(name)
    }

    /// Filters events that have the provided attribute name/value,
    /// in addition to the tag name.
    pub fn attr<NP, VP>(self, name: NP, value: VP) -> Attr<P, NP, VP>
    where
        NP: BytePat,
        VP: BytePat
    {
        Attr {
            tag: self,
            name,
            value
        }
    }
    
    shorthand!(class);
    shorthand!(id);
    shorthand!(name);
}

/// Returns a [pattern](BytePat) that checks whether
/// an attribute's value is present in a space-separated
/// list, such as a `class` list.
///
/// Intended to be used with [Tag::attr](Tag::attr) or
/// one of the shorthands that use it.
pub fn has(value: impl BytePat) -> impl BytePat {
    pat::contains(b' ', value)
}

impl < 'evt, P: BytePat> Filter<'evt> for Tag<P> {
    fn start(&mut self, event: &Event<'evt>) -> bool {
        match event {
            Event::Start(tag) if self.matches(tag.name()) => {
                self.opened = 1;
                true
            },
            _ => false
        }
    }

    fn end(&mut self, event: &Event<'evt>) -> bool {
        match event {
            Event::End(tag) if self.matches(tag.name()) => {
                self.opened -= 1;
                self.opened == 0
            },
            Event::Start(tag) if self.matches(tag.name()) => {
                self.opened += 1;
                false
            }
            _ => false
        }
    }
}

/// Filter created by the [Tag::attr](Tag::attr) function. 
pub struct Attr<TP, NP, VP> {
    tag: Tag<TP>,
    name: NP,
    value: VP
}

impl <TP, NP, VP> Attr<TP, NP, VP>
where 
    TP: BytePat,
    NP: BytePat,
    VP: BytePat
{
    fn matches(&self, attr: &attributes::Attribute) -> bool {
        self.name.matches(attr.key)
            && self.value.matches(&attr.value) 
    }
}

impl <'evt, TP, NP, VP> Filter<'evt> for Attr<TP, NP, VP>
where 
    TP: BytePat,
    NP: BytePat,
    VP: BytePat
{
    fn start(&mut self, event: &Event<'evt>) -> bool {
        match event {
            Event::Start(tag) if self.tag.matches(tag.name()) && tag
                .html_attributes()
                .with_checks(false)
                .filter_map(Result::ok)
                .any(|attr| self.matches(&attr))
            => {
                self.tag.opened = 1;
                true
            },
            _ => false
        }
    }

    fn end(&mut self, event: &Event<'evt>) -> bool {
        self.tag.end(event)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tag() {
        let events = [
            Event::Comment(BytesText::from_plain_str("ac")),
            Event::Start(BytesStart::borrowed_name(b"tag")),
            Event::Start(BytesStart::borrowed_name(b"tag")),
            Event::Comment(BytesText::from_plain_str("ab")),
            Event::End(BytesEnd::borrowed(b"tag")),
            Event::End(BytesEnd::borrowed(b"tag")),
        ];

        let mut events = events.iter();
        let mut next = || events.next().unwrap();

        let mut filter = super::tag("tag");

        assert!(!filter.start(next()));
        assert!(filter.start(next()));
        assert!(!filter.end(next()));
        assert!(!filter.end(next()));
        assert!(!filter.end(next()));
        assert!(filter.end(next()));
    }
}
