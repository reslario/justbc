use crate::state::{Core, WidgetState};

macro_rules! no_ops {
    () => {
        fn selection_down(&self, _widgets: &mut WidgetState) {}
        fn selection_up(&self, _widgets: &mut WidgetState) {}
        fn confirm(
            &self,
            _core: &mut Core,
            _widgets: &mut WidgetState,
        ) -> Option<super::ExploreState> {
            None
        }
    };
}

pub struct Blank;

impl super::Explore for Blank {
    no_ops!();
}

pub struct Loading;

impl super::Explore for Loading {
    no_ops!();
}
