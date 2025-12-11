use {
    crate::state::{explore::ExploreState, Core, WidgetState},
    bandcamp_api::data::{
        fans::{Fan, FanArgs},
        outlets::Outlet,
        releases::{Release, ReleaseArgs, ReleaseKind},
        search::SearchResult,
    },
};

impl super::Explore for super::Search {
    fn selection_down(&self, widgets: &mut WidgetState) {
        if super::can_select_down(widgets.nav.selected(), self.results.len()) {
            widgets.nav.selection_down()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<ExploreState> {
        widgets.nav.selected().map(|idx| {
            match &self.results[idx] {
                SearchResult::Outlet(o) => core.fetcher.query::<Outlet, _>(&o.id),
                SearchResult::Album(a) => {
                    let args = ReleaseArgs {
                        id: a.id,
                        kind: ReleaseKind::Album,
                        outlet: a.artist_id,
                    };

                    core.fetcher.query::<Release, _>(&args)
                }
                SearchResult::Track(t) => {
                    let args = ReleaseArgs {
                        id: t.id,
                        kind: ReleaseKind::Track,
                        outlet: t.artist_id,
                    };

                    core.fetcher.query::<Release, _>(&args)
                }
                SearchResult::Fan(f) => {
                    let args = FanArgs {
                        id: f.id,
                        start: 0,
                        count: FanArgs::DEFAULT_COUNT,
                    };

                    core.fetcher.query::<Fan, _>(&args)
                }
            };

            ExploreState::loading()
        })
    }
}
