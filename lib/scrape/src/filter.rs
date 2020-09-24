//! Filters for restricting a scraper's output.

use quick_xml::events::*;

/// Defines the interface for a filter, which consists
/// of two functions that match the start and end of
/// a set of [events](crate::Event).
pub trait Filter<'evt> {
    /// Determines whether an event marks t
    fn start(&mut self, event: &Event<'evt>) -> bool;
    fn end(&mut self, event: &Event<'evt>) -> bool;
}

/// Filter created by the [tag](tag) function. 
#[derive(Clone, Copy)]
pub struct Tag<'a> {
    name: &'a str,
    opened: usize
}

/// Filters for events that match the provided tag name.
pub fn tag(name: &str) -> Tag {
    Tag { name, opened: 0 }
}

/// Filters events that match the `div` tag.
///
/// Shorthand for `.tag("div")`
pub fn div() -> Tag<'static> {
    tag("div")
}

impl <'a> Tag<'a> {
    fn matches(&self, name: &[u8]) -> bool {
        name == self.name.as_bytes()
    }

    /// Filters events that have the provided attribute name/value,
    /// in addition to the tag name.
    pub fn attr(self, name: &'a str, value: &'a str) -> Attr<'a> {
        Attr {
            tag: self,
            name,
            value
        }
    }
    
    /// Filters events that have the provided class,
    /// in addition to the tag name.
    /// 
    /// Shorthand for `.attr("class", <value>)`
    pub fn class(self, value: &'a str) -> Attr<'a> {
        self.attr("class", value)
    }
}

impl <'a, 'evt> Filter<'evt> for Tag<'a> {
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
pub struct Attr<'a> {
    tag: Tag<'a>,
    name: &'a str,
    value: &'a str
}

impl Attr<'_> {
    fn matches(&self, attr: &attributes::Attribute) -> bool {
        attr.key == self.name.as_bytes()
            && attr.value == self.value.as_bytes()
    }
}

impl <'a, 'evt> Filter<'evt> for Attr<'a> {
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
