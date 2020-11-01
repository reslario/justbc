use {
    std::ops,
    tui::{
        layout,
        buffer,
        widgets::{Widget, StatefulWidget}
    }
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Stateful<W: StatefulWidget> {
    state: W::State
}

impl <W> Default for Stateful<W>
where
    W: StatefulWidget,
    W::State: Default
{
    fn default() -> Self {
        Stateful { state: <_>::default() }
    }
}

impl <W: StatefulWidget> Stateful<W> {
    pub fn new(state: W::State) -> Stateful<W> {
        Stateful { state }
    }

    pub fn render(&mut self, widget: W) -> impl Widget + '_ {
        Render {
            state: &mut self.state,
            widget
        }
    }
}

impl <W: StatefulWidget> ops::Deref for Stateful<W> {
    type Target = W::State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl <W: StatefulWidget> ops::DerefMut for Stateful<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

struct Render<'state, W: StatefulWidget> {
    state: &'state mut W::State,
    widget: W
}

impl <'state, W: StatefulWidget> Widget for Render<'state, W> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        self.widget.render(area, buf, self.state)
    }
}
