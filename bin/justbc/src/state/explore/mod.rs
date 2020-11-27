mod empty;
mod search;
mod outlet;
mod release;

pub use empty::*;

use super::*;

#[enumerate::enumerate(ExploreState: Blank, Loading, Search, Outlet, Release)]
pub trait Explore {
    fn selection_down(&self, widgets: &mut WidgetState);

    fn selection_up(&self, widgets: &mut WidgetState) {
        if can_select_up(widgets.nav.selected()) {
            widgets.nav.selection_up()
        }
    }

    fn confirm(&self, core: &mut Core, widgets: &mut WidgetState) -> Option<ExploreState>;
}

impl ExploreState {
    pub fn blank() -> ExploreState {
        ExploreState::Blank(Blank)
    }

    pub fn loading() -> ExploreState {
        ExploreState::Loading(Loading)
    }
}

impl Default for ExploreState {
    fn default() -> Self {
        ExploreState::blank()
    }
}
