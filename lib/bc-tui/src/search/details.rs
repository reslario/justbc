use {
    builder::builder_method,
    gen_tui::style::StyleExt,
    bandcamp_api::data::{
        search::*,
        common::Date
    },
    tui::{
        style::Style,
        layout::Rect,
        buffer::Buffer,
        text::{Text, Span, Spans},
        widgets::{Widget, Paragraph, Wrap}
    }
};

pub struct ResultDetails<'a> {
    result: &'a SearchResult,
    style: Style
}

impl <'a> ResultDetails<'a> {
    pub fn new(result: &'a SearchResult) -> ResultDetails<'a> {
        ResultDetails {
            result,
            style: <_>::default()
        }
    }

    builder_method! {
        pub style: Style
    }
}

impl <'a> Widget for ResultDetails<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(text(self.result))
            .style(self.style)
            .wrap(Wrap { trim: true })
            .render(area, buf)
    }
}

pub fn text(result: &SearchResult) -> Text {
    use SearchResult as R;

    match result {
        R::Track(track) => titled("TRACK", track_text(track)),
        R::Album(album) => titled("ALBUM", album_text(album)),
        R::Label(label) => titled("LABEL", label_text(label)),
        R::Artist(artist) => titled("ARTIST", artist_text(artist))
    }
}

macro_rules! iter {
    ($head:expr, $($tail:tt)+) => {
        std::iter::once($head)
            .chain(iter!($($tail)+))
    };

    ($last:expr) => {
        std::iter::once($last)
    }
}

pub fn titled<'a>(title: &'a str, rest: impl Iterator<Item = Spans<'a>>) -> Text<'a> {
    let mut text = Text::from(vec![
        Span::styled(title, Style::default().bold()).into(),
        Spans::from("")
    ]);

    text.extend(rest);

    text
}

fn track_text(track: &Track) -> impl Iterator<Item = Spans> {
    iter![
        underlined(&track.heading.title).into(),
        source(&track.source),
        released(track.released),
        tags(track.tags.as_ref())
    ]
}

fn album_text(album: &Album) -> impl Iterator<Item = Spans> {
    iter![
        underlined(&album.heading.title).into(),
        by(&album.by),
        released(album.released),
        tags(album.tags.as_ref())
    ]
}

fn label_text(label: &Label) -> impl Iterator<Item = Spans> {
    iter![
        underlined(&label.heading.title).into(),
        location(label.sub_heading.as_deref()),
        tags(label.tags.as_ref())
    ]
}

fn artist_text(artist: &Artist) -> impl Iterator<Item = Spans> {
    iter![
        underlined(&artist.heading.title).into(),
        location(artist.sub_heading.as_deref()),
        genre(artist.genre.as_deref()),
        tags(artist.tags.as_ref())
    ]
}

fn source(source: &Source) -> Spans {
    let mut spans = source.from
        .as_deref()
        .map(|from| vec![
            bold("from "),
            from.into()
        ])
        .unwrap_or_default();
    
    if !spans.is_empty() {
        spans.push(Span::raw(" "))
    }

    spans.extend(by(&source.by).0);

    spans.into()
}

fn by(by: &str) -> Spans {
    vec![
        bold("by ").into(),
        Span::raw(by).into()
    ].into()
}

fn released(date: Date) -> Spans<'static> {
    vec![
        bold("released "),
        date.fmt_long().to_string().into()
    ].into()
}

fn tags(tags: Option<&Tags>) -> Spans {
    tags.map(|tags| vec![
        bold("tags: "),
        tags.to_string().into()
    ])
    .unwrap_or_default()
    .into()
}

fn location(location: Option<&str>) -> Spans {
    location
        .map(|location| vec![
            bold("location: "),
            location.into()
        ])
        .unwrap_or_default()
        .into()
}

fn genre(genre: Option<&str>) -> Spans {
    genre
        .map(|genre| vec![
            bold("genre: "),
            genre.into()
        ])
        .unwrap_or_default()
        .into()
}

fn bold<'a>(text: impl Into<std::borrow::Cow<'a, str>>) -> Span<'a> {
    Span::styled(text, Style::default().bold())
}

fn underlined<'a>(text: impl Into<std::borrow::Cow<'a, str>>) -> Span<'a> {
    Span::styled(text, Style::default().underlined())
}
