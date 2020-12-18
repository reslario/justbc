use {
    bandcamp_api::data::releases::{Release, ReleaseArgs},
    crate::state::{
        Core,
        WidgetState,
        explore::ExploreState
    },
};

impl super::Explore for super::Outlet {
    fn selection_down(&self, widgets: &mut WidgetState) {
        if super::can_select_down(widgets.nav.selected(), self.discography.len()) {
            widgets.nav.selection_down()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<super::ExploreState> {
        widgets.nav
            .selected()
            .map(|idx| {
                let release = &self.discography[idx];

                let args = ReleaseArgs {
                    id: release.id,
                    kind: release.kind,
                    outlet: self.info.id,
                };

                core.fetcher.query::<Release, _>(&args);

                ExploreState::loading()
            })
    }
}
