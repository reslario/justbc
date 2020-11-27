use {
    builder::builder_methods,
    super::{
        search::*,
        outlets::*,
        releases::*
    },
    gen_tui::{
        style::StyleExt,
        layout::RectExt,
        widgets::{StatefulWidgetExt, Spinner, SpinnerState, TextInput, TextInputState},
    },
    bandcamp_api::data::{
        outlets::*,
        search::Search,
        releases::Release
    },
    tui::{
        text::Span,
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::{StatefulWidget, Tabs}
    }
};

#[derive(Copy, Clone)]
enum Show<'a> {
    Library,
    Explore(Explore<'a>)
}

impl <'a> Default for Show<'a> {
    fn default() -> Self {
        Show::Library
    }
}

#[derive(Copy, Clone)]
enum Explore<'a> {
    Blank,
    Loading,
    Search(&'a Search),
    Outlet(&'a Outlet),
    Release(&'a Release)
}

#[derive(Default)]
pub struct NavView<'a> {
    show: Show<'a>,
    style: Style,
    highlight_style: Style
}

impl <'a> NavView<'a> {
    pub fn library(self) -> Self {
        Self { show: Show::Library, ..self }
    }

    pub fn blank(self) -> Self {
        Self { show: Show::Explore(Explore::Blank), ..self }
    }

    pub fn loading(self) -> Self {
        Self { show: Show::Explore(Explore::Loading), ..self }
    }

    pub fn search(self, search: &'a Search) -> Self {
        Self { show: Show::Explore(Explore::Search(search)), ..self }
    }

    pub fn outlet(self, outlet: &'a Outlet) -> Self {
        Self { show: Show::Explore(Explore::Outlet(outlet)), ..self }
    }

    pub fn release(self, release: &'a Release) -> Self {
        Self { show: Show::Explore(Explore::Release(release)), ..self }
    }

    builder_methods! {
        pub style: Style;
        pub highlight_style: Style
    }

    const TABS: [&'static str; 2] = ["Library", "Explore"];

    fn draw_tabs(&self, area: Rect, buf: &mut Buffer) -> Rect {
        use tui::widgets::Widget;

        let width = Self::TABS
            .iter()
            .copied()
            .map(str::len)
            .sum::<usize>()
            + 3;

        let center_offset = (width / 2 + 1) as u16;

        let (left, _) = area.split_ratio_x(0.5);
        let tab_area = left
            .from_right(center_offset)
            .grow_right(center_offset)
            .from_top(1);

        let titles = ["Library", "Explore"]
            .iter()
            .cloned()
            .map(Span::raw)
            .map(<_>::into)
            .collect();

        let selected = match self.show {
            Show::Library => 0,
            Show::Explore(_) => 1
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
            }
        }

        match self.show {
            Show::Library => todo!(),
            Show::Explore(xp) => match xp {
                Explore::Blank => state.blank(),
                Explore::Loading => self.draw_loading(area, buf, state.spinner()),
                Explore::Search(search) => draw!(search, Self::draw_search, NavViewState::results),
                Explore::Outlet(outlet) => draw!(outlet, Self::draw_outlet, NavViewState::outlet),
                Explore::Release(release) => draw!(release, Self::draw_release, NavViewState::release)
            }
        }
    }

    fn draw<'v, T, W, WF, SF>(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut NavViewState,
        val: &'v T,
        mut wf: WF,
        mut sf: SF
    )
    where
        W: StatefulWidget + StatefulWidgetExt + 'v,
        WF: FnMut(&Self, &'v T) -> W,
        SF: for<'s> FnMut(&'s mut NavViewState) -> &'s mut W::State
    {
        wf(self, val)
            .scrollable()
            .scroll_y(state.scroll)
            .render(area, buf, sf(state))
    }

    fn draw_loading(&self, area: Rect, buf: &mut Buffer, state: &mut SpinnerState) {
        let (left, _) = area.split_ratio_x(0.5);
        let (top_left, _) = left.split_ratio_y(0.5);

        let area = top_left
            .from_right(2)
            .grow_right(2)
            .from_bottom(1)
            .grow_bottom(1);

        Spinner::default()
            .render(area, buf, state)
    }

    fn draw_search<'s>(&self, search: &'s Search) -> ResultView<'s> {
        ResultView::new(&search.results)
            .style(self.style)
            .highlight_style(self.highlight_style)
    }

    fn draw_outlet<'o>(&self, outlet: &'o Outlet) -> OutletView<'o> {
        OutletView::new(outlet)
            .style(self.style)
            .highlight_style(self.highlight_style)
    }

    fn draw_release<'r>(&self, release: &'r Release) -> ReleaseView<'r> {
        ReleaseView::new(release)
            .style(self.style)
    }
}

#[derive(Default)]
pub struct NavViewState {
    pub input: TextInputState,
    pub results: ResultViewState,
    pub release: ReleaseViewState,
    pub outlet: OutletViewState,
    spinner: SpinnerState,
    scroll: u16
}

impl NavViewState {
    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll += amount;
    }

    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll -= amount;
    }

    fn unscroll(&mut self) {
        self.scroll = 0;
    }

    fn blank(&mut self) {
        let input = std::mem::take(&mut self.input);
        *self = NavViewState { input, ..<_>::default() }
    }

    fn spinner(&mut self) -> &mut SpinnerState {
        self.unscroll();

        reset(
            &mut self.results,
            &mut self.release,
            &mut self.outlet
        );

        &mut self.spinner
    }

    fn results(&mut self) -> &mut ResultViewState {
        self.unscroll();

        reset(
            &mut self.spinner,
            &mut self.release,
            &mut self.outlet
        );

        &mut self.results
    }
    
    fn release(&mut self) -> &mut ReleaseViewState {
        self.unscroll();

        reset(
            &mut self.spinner,
            &mut self.results,
            &mut self.outlet
        );

        &mut self.release
    }
    
    fn outlet(&mut self) -> &mut OutletViewState {
        self.unscroll();
        
        reset(
            &mut self.spinner,
            &mut self.release,
            &mut self.results
        );

        &mut self.outlet
    }
}

fn reset(a: &mut impl Default, b: &mut impl Default, c: &mut impl Default) {
    *a = <_>::default();
    *b = <_>::default();
    *c = <_>::default();
}

impl <'a> StatefulWidget for NavView<'a> {
    type State = NavViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.draw_tabs(area, buf);
        let area = self.draw_search_bar(area, buf, state);
        self.draw_body(area, buf, state)
    }
}
