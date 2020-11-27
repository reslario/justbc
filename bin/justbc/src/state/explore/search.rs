use {
    crate::state::{
        Core,
        WidgetState,
        explore::ExploreState
    },
    bandcamp_api::data::{
        outlets::Outlet,
        releases::Release,
        search::{self, SearchResult}
    }
};

impl super::Explore for super::Search {
    fn selection_down(&self, widgets: &mut WidgetState) {
        if super::can_select_down(widgets.nav.selected(), self.results.len()) {
            widgets.nav.selection_down()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<ExploreState> {
        widgets.nav
            .selected()
            .map(|idx| {
                match &self.results[idx] {
                    SearchResult::Artist(search::Artist { heading, .. })
                        | SearchResult::Label(search::Label { heading, .. }) 
                            => core.fetcher.query::<Outlet, _, _>(&heading.url),
                    SearchResult::Album(search::Album { heading, .. })
                        | SearchResult::Track(search::Track { heading, .. })
                            => core.fetcher.query::<Release, _, _>(&heading.url),
                };

                ExploreState::loading()
            })
    }
}
