use {
    bandcamp_api::data::{
        fans::{Fan, FanArgs},
        releases::{Release, ReleaseArgs}
    },
    crate::state::{
        Core,
        WidgetState,
        explore::ExploreState
    },
};

impl super::Explore for super::Fan {
    fn selection_down(&self, widgets: &mut WidgetState) {
        if super::can_select_down(widgets.nav.selected(), self.collection.len() + 1) {
            widgets.nav.selection_down()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<super::ExploreState> {
        widgets.nav
            .selected()
            .and_then(|idx| {
                if let Some(collected) = self.collection.get(idx) {
                    let args = ReleaseArgs {
                        id: collected.id,
                        kind: collected.kind,
                        outlet: collected.artist_id,
                    };

                    core.fetcher.query::<Release, _>(&args);

                    Some(ExploreState::loading())
                } else {
                    let args = FanArgs {
                        id: self.id,
                        start: self.collection.len() as _,
                        count: FanArgs::DEFAULT_COUNT,
                    };

                    core.fetcher.query::<Fan, _>(&args);
                    widgets.nav.fan().set_loading(true);
                    None
                }
            })
    }
}
