use {
    crate::state::{explore::ExploreState, Core, Focus, State, WidgetState},
    bandcamp_api::data::releases::Release,
};

impl super::Explore for Release {
    fn selection_down(&self, widgets: &mut WidgetState) {
        if super::can_select_down(widgets.nav.selected(), self.tracks.len()) {
            widgets.nav.selection_down()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<ExploreState> {
        if let Some(idx) = widgets.nav.selected() {
            core.set_release(self.clone(), idx);
            widgets.release.play(idx);
            widgets.release.select(idx.into());
            State::sync_focus(Focus::Release, core, widgets);
        }

        None
    }
}
