use {
    super::{fans::*, outlets::*, releases::*, search::*},
    bandcamp_api::data::{fans::Fan, outlets::*, releases::Release, search::Search},
    builder::builder_methods,
    gen_tui::{
        layout::RectExt,
        style::StyleExt,
        widgets::{Spinner, SpinnerState, StatefulWidgetExt, TextInput, TextInputState},
    },
    tui::{
        buffer::Buffer,
        layout::Rect,
        style::Style,
        text::Span,
        widgets::{ListState, StatefulWidget, Tabs},
    },
};

#[derive(Copy, Clone, Default)]
enum Show<'a> {
    #[default]
    Library,
    Explore(Explore<'a>),
}

#[derive(Copy, Clone)]
enum Explore<'a> {
    Blank,
    Loading,
    Fan(&'a Fan),
    Search(&'a Search),
    Outlet(&'a Outlet),
    Release(&'a Release),
}

#[derive(Default)]
pub struct NavView<'a> {
    show: Show<'a>,
    style: Style,
    highlight_style: Style,
}

impl<'a> NavView<'a> {
    pub fn library(self) -> Self {
        Self {
            show: Show::Library,
            ..self
        }
    }

    pub fn blank(self) -> Self {
        Self {
            show: Show::Explore(Explore::Blank),
            ..self
        }
    }

    pub fn loading(self) -> Self {
        Self {
            show: Show::Explore(Explore::Loading),
            ..self
        }
    }

    pub fn fan(self, fan: &'a Fan) -> Self {
        Self {
            show: Show::Explore(Explore::Fan(fan)),
            ..self
        }
    }

    pub fn search(self, search: &'a Search) -> Self {
        Self {
            show: Show::Explore(Explore::Search(search)),
            ..self
        }
    }

    pub fn outlet(self, outlet: &'a Outlet) -> Self {
        Self {
            show: Show::Explore(Explore::Outlet(outlet)),
            ..self
        }
    }

    pub fn release(self, release: &'a Release) -> Self {
        Self {
            show: Show::Explore(Explore::Release(release)),
            ..self
        }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }

    const TABS: [&'static str; 2] = ["Library", "Explore"];

    fn draw_tabs(&self, area: Rect, buf: &mut Buffer) -> Rect {
        use tui::widgets::Widget;

        let width = Self::TABS.iter().copied().map(str::len).sum::<usize>() + 3;

        let center_offset = (width / 2 + 1) as u16;

        let (left, _) = area.split_ratio_x(0.5);
        let tab_area = left
            .scale_from_right(center_offset)
            .grow_right(center_offset)
            .scale_from_top(1);

        let titles = ["Library", "Explore"]
            .iter()
            .cloned()
            .map(Span::raw)
            .map(<_>::into)
            .collect();

        let selected = match self.show {
            Show::Library => 0,
            Show::Explore(_) => 1,
        };

        Tabs::new(titles)
            .divider("/")
            .select(selected)
            .style(self.style.bold())
            .highlight_style(self.highlight_style)
            .render(tab_area, buf);

        area.shrink_top(2)
    }

    fn draw_search_bar(&self, area: Rect, buf: &mut Buffer, state: &mut NavViewState) -> Rect {
        TextInput::default()
            .prompt("🔍")
            .style(self.style)
            .render(area, buf, &mut state.input);

        area.shrink_top(2)
    }

    fn draw_body(&self, area: Rect, buf: &mut Buffer, state: &mut NavViewState) {
        macro_rules! draw {
            ($val:expr, $wf:expr, $sf:expr) => {
                self.draw(area, buf, state, $val, $wf, $sf)
            };
        }

        match self.show {
            Show::Library => todo!(),
            Show::Explore(xp) => match xp {
                Explore::Blank => state.blank(),
                Explore::Loading => self.draw_loading(area, buf, state.spinner()),
                Explore::Fan(fan) => draw!(fan, Self::draw_fan, NavViewState::fan),
                Explore::Search(search) => draw!(search, Self::draw_search, NavViewState::results),
                Explore::Outlet(outlet) => draw!(outlet, Self::draw_outlet, NavViewState::outlet),
                Explore::Release(release) => {
                    draw!(release, Self::draw_release, NavViewState::release)
                }
            },
        }
    }

    fn draw<'v, T, W, WF, SF>(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut NavViewState,
        val: &'v T,
        mut wf: WF,
        mut sf: SF,
    ) where
        W: StatefulWidget + StatefulWidgetExt + 'v,
        WF: FnMut(&Self, &'v T) -> W,
        SF: for<'s> FnMut(&'s mut NavViewState) -> &'s mut W::State,
    {
        wf(self, val)
            .scrollable()
            .scroll_y(state.scroll)
            .render(area, buf, sf(state))
    }

    fn draw_loading(&self, area: Rect, buf: &mut Buffer, state: &mut SpinnerState) {
        Spinner::default().render(area.centered(4, 2), buf, state)
    }

    fn draw_fan<'f>(&self, fan: &'f Fan) -> FanView<'f> {
        FanView::new(fan)
            .style(self.style)
            .highlight_style(self.highlight_style)
    }

    fn draw_search<'s>(&self, search: &'s Search) -> ResultList<'s> {
        ResultList::new(&search.results)
            .style(self.style)
            .highlight_style(self.highlight_style)
    }

    fn draw_outlet<'o>(&self, outlet: &'o Outlet) -> OutletView<'o> {
        OutletView::new(outlet)
            .style(self.style)
            .highlight_style(self.highlight_style)
    }

    fn draw_release<'r>(&self, release: &'r Release) -> ReleaseView<'r> {
        ReleaseView::new(release).style(self.style)
    }
}

#[derive(Default)]
enum BodyState {
    #[default]
    Blank,
    Fan(FanViewState),
    Results(ResultListState),
    Release(ReleaseViewState),
    Outlet(OutletViewState),
    Spinner(SpinnerState),
}

#[derive(Default)]
pub struct NavViewState {
    pub input: TextInputState,
    body: BodyState,
    scroll: u16,
}

macro_rules! get_body {
    ($var:ident. $field:ident, $variant:ident) => {
        match &mut $var.$field {
            BodyState::$variant(state) => state,
            body => {
                $var.scroll = 0;
                *body = BodyState::$variant(<_>::default());
                match body {
                    BodyState::$variant(state) => state,
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl NavViewState {
    fn list_mut(&mut self) -> Option<&mut ListState> {
        match &mut self.body {
            BodyState::Blank | BodyState::Spinner(_) => return None,
            BodyState::Outlet(o) => o,
            BodyState::Release(r) => &mut *r,
            BodyState::Results(r) => r,
            BodyState::Fan(f) => &mut f.collection,
        }
        .into()
    }

    fn list(&self) -> Option<&ListState> {
        match &self.body {
            BodyState::Blank | BodyState::Spinner(_) => return None,
            BodyState::Outlet(o) => o,
            BodyState::Release(r) => r,
            BodyState::Results(r) => r,
            BodyState::Fan(f) => &f.collection,
        }
        .into()
    }

    pub fn selected(&self) -> Option<usize> {
        self.list()?.selected()
    }

    pub fn selection_down(&mut self) {
        if let Some(list) = self.list_mut() {
            let index = list.selected().map(|idx| idx + 1).unwrap_or_default();

            list.select(index.into())
        }
    }

    pub fn selection_up(&mut self) {
        if let Some(list) = self.list_mut() {
            list.select(list.selected().map(|s| s.saturating_sub(1)))
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1)
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1)
    }

    fn blank(&mut self) {
        self.body = BodyState::Blank;
        self.scroll = 0
    }

    pub fn spinner(&mut self) -> &mut SpinnerState {
        get_body!(self.body, Spinner)
    }

    pub fn fan(&mut self) -> &mut FanViewState {
        get_body!(self.body, Fan)
    }

    pub fn results(&mut self) -> &mut ResultListState {
        get_body!(self.body, Results)
    }

    pub fn release(&mut self) -> &mut ReleaseViewState {
        get_body!(self.body, Release)
    }

    pub fn outlet(&mut self) -> &mut OutletViewState {
        get_body!(self.body, Outlet)
    }
}

impl<'a> StatefulWidget for NavView<'a> {
    type State = NavViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_tabs(area, buf);
        let area = self.draw_search_bar(area, buf, state);
        self.draw_body(area, buf, state)
    }
}
