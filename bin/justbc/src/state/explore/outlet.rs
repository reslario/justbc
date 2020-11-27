use {
    bandcamp_api::data::releases::Release,
    crate::state::{
        Core,
        WidgetState,
        explore::ExploreState
    },
};

impl super::Explore for super::Outlet {
    fn selection_down(&self, widgets: &mut WidgetState) {
        let len = self.featured.len() 
            + self.releases.len();

        if super::can_select_down(widgets.nav.selected(), len) {
            widgets.nav.selection_down()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<super::ExploreState> {
        widgets.nav
            .selected()
            .map(|idx| {
                let release = self
                .featured
                .get(idx)
                .map(|feat| &**feat)
                .unwrap_or_else(|| &self.releases[idx - self.featured.len()].info);

                core.fetcher.query::<Release, _, _>(&release.url);

                ExploreState::loading()
            })
    }
}
