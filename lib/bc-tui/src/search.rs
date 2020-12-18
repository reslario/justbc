use {
    crate::symbols,
    std::borrow::Cow,
    builder::builder_methods,
    bandcamp_api::data::{
        outlets::OutletKind,
        search::{SearchResult, Track, Album},
    },
    tui::{
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::{StatefulWidget, List, ListState, ListItem}
    }
};

pub struct ResultList<'a> {
    results: &'a [SearchResult],
    style: Style,
    highlight_style: Style
}

impl <'a> ResultList<'a> {
    pub fn new(results: &'a [SearchResult]) -> ResultList<'a> {
        ResultList {
            results,
            style: <_>::default(),
            highlight_style: <_>::default(),
        }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }
}

#[derive(Default)]
pub struct ResultListState {
    list: ListState
}

impl std::ops::Deref for ResultListState {
    type Target = ListState;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl std::ops::DerefMut for ResultListState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl <'a> StatefulWidget for ResultList<'a> {
    type State = ResultListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = self
            .results
            .iter()
            .map(list_item)
            .collect::<Vec<_>>();

        List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .render(area, buf, &mut state.list)
    }
}

fn list_item(result: &SearchResult) -> ListItem {
    ListItem::new(
        format!("{} {}", icon(result), text(result))
    )
}

fn icon(result: &SearchResult) -> char {
    match result {
        SearchResult::Track(_) => symbols::TRACK,
        SearchResult::Album(_) => symbols::ALBUM,
        SearchResult::Fan(_) => symbols::FAN,
        SearchResult::Outlet(o) => match o.kind {
            OutletKind::Artist => symbols::ARTIST,
            OutletKind::Label => symbols::LABEL
        },
    }
}

fn text(result: &SearchResult) -> Cow<str> {
    use SearchResult as R;

    match result {
        R::Track(track) => track_text(track).into(),
        R::Album(album) => album_text(album).into(),
        R::Outlet(outlet) => outlet.name.as_str().into(),
        R::Fan(fan) => fan.name.as_str().into()
    }
}

fn track_text(track: &Track) -> String {
    crate::fmt_release(&track.artist, &track.name)
}

fn album_text(album: &Album) -> String {
    crate::fmt_release(&album.artist, &album.name)
}
