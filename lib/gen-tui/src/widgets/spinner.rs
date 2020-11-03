use {
    std::iter,
    builder::builder_method,
    tui::{
        style::Style,
        layout::Rect,
        buffer::Buffer,
        widgets::StatefulWidget
    }
};

#[derive(Default)]
pub struct Spinner {
    style: Style
}

impl Spinner {
    builder_method! {
        pub style: Style
    }
}

type Symbol = ([usize; 2], char);
type Symbols = iter::Copied<std::slice::Iter<'static, Symbol>>;

type Iter = iter::Cycle<iter::Zip<
    Symbols,
    iter::Chain<iter::Skip<Symbols>, std::option::IntoIter<Symbol>>>
>;
pub struct SpinnerState {
    symbols: Iter
}

const SYMBOLS: &[Symbol] = &[
    ([1, 0], '─'),
    ([2, 0], '╮'),
    ([2, 1], '╯'),
    ([1, 1], '─'),
    ([0, 1], '╰'),
    ([0, 0], '╭'),
];

impl Default for SpinnerState {
    fn default() -> Self {
        SpinnerState {
            symbols: symbols()
                .zip(symbols()
                    .skip(1)
                    .chain(symbols().next())
                ).cycle()
        }
    }
}

fn symbols() -> Symbols {
    SYMBOLS.iter().copied()
}

const WIDTH: u16 = 3;
const HEIGHT: u16 = 2;

type Cells = [[char; WIDTH as _]; HEIGHT as _];

impl StatefulWidget for Spinner {
    type State = SpinnerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width < WIDTH || area.height < HEIGHT { return }

        let mut cells = [[' '; WIDTH as _]; HEIGHT as _];

        let (a, b) = state.symbols.next().unwrap();

        insert(a, &mut cells);
        insert(b, &mut cells);

        for (y, row) in cells.iter().enumerate() {
            for (x, sym) in row.iter().enumerate() {
                buf.get_mut(x as _, y as _)
                    .set_char(*sym)
                    .set_style(self.style);
            }
        }
    }
}

fn insert(([x, y], sym): Symbol, cells: &mut Cells) {
    cells[y][x] = sym
}
