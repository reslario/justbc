use {
    builder::builder_methods,
    tui::{
        buffer::Buffer,
        layout::Rect,
        style::Style,
        widgets::{Block, Widget},
    },
};

/// A customisable progress bar.
#[derive(Default)]
pub struct ProgressBar<'a, S> {
    symbols: S,
    progress: f32,
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a, S: AsRef<[char]>> ProgressBar<'a, S> {
    pub fn labeled<T>() -> super::Labeled<'a, T, S>
    where
        T: Default,
        S: Default,
    {
        <_>::default()
    }

    builder_methods! {
        /// Sets the symbols this `ProgressBar` uses.
        ///
        /// `ProgressBar`s support drawing different symbols to fill
        /// a cell depending on completion, which can make it look smoother.
        /// The symbols should be in ascending "fullness". So if you wanted
        /// to, for example, use [Block Elements](https://en.wikipedia.org/wiki/Block_Elements),
        /// you'd want to order them like this:
        ///
        /// ```ignore
        /// .symbols(['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'])
        /// ```
        pub symbols: S;

        /// Sets the progress of this bar (0 - 1).
        /// Will be clamped automatically.
        pub progress: f32 => progress.clamp(0., 1.);

        pub block: Block<'a> => block.into();
        pub style: Style
    }
}

impl<'a, S: AsRef<[char]>> Widget for ProgressBar<'a, S> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use std::iter;

        let area = self
            .block
            .map_or(area, crate::widgets::rendered_block(area, buf));

        buf.set_style(Rect { height: 1, ..area }, self.style);

        let len = f32::from(area.width) * self.progress;

        let (full, partial) = symbols(self.symbols, len);

        let end = area.x + len.ceil() as u16;

        (area.x..end)
            .zip(iter::repeat(full))
            .chain(iter::once((end, partial)))
            .for_each(|(x, sym)| set_char(buf, x, area.y, sym));
    }
}

fn symbols(all: impl AsRef<[char]>, len: f32) -> (char, char) {
    use std::ops::Mul;

    let symbols = all.as_ref();

    let full = symbols.last().copied().unwrap_or('-');

    let index = len.fract().mul(symbols.len() as f32).round() as usize;

    let partial = if index == 0 { ' ' } else { symbols[index - 1] };

    (full, partial)
}

fn set_char(buf: &mut Buffer, x: u16, y: u16, c: char) {
    buf.get_mut(x, y).set_char(c);
}
