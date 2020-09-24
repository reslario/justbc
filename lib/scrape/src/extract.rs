//! Functions that extract specific values from [Events](crate::Event).

use quick_xml::events::*;

/// Extracts the text from a [text event](crate::Event::Text).
/// Returns None if the event is not a [text event](crate::Event::Text).
pub fn text<'evt>(event: Event<'evt>) -> Option<crate::Result<String>> {
    match event {
        Event::Text(text) => text
            .unescaped()
            .and_then(to_string)
            .into(),
        _ => None
    }
}

/// Extracts the value of an attribute from a [start event](crate::Event::Start).
/// Returns None if the event is not a [start event](crate::Event::Start).
pub fn attr(name: impl AsRef<[u8]>) -> impl for<'evt> FnMut(Event<'evt>) -> Option<crate::Result<String>> {
    move |event| match event {
        Event::Start(tag) => tag
            .html_attributes()
            .with_checks(false)
            .filter_map(Result::ok)
            .find_map(|attr| if attr.key == name.as_ref() {
                attr_value(attr).into()
            } else {
                None
            }),
        _ => None
    }
}

fn attr_value(attr: attributes::Attribute) -> crate::Result<String> {
    attr.unescaped_value()
        .and_then(to_string)
}

fn to_string(bytes: std::borrow::Cow<[u8]>) -> crate::Result<String> {
    String::from_utf8(bytes.into())
        .map_err(|e| e.utf8_error())
        .map_err(crate::Error::Utf8)
}
