use {
    builder::builder_method,
    std::{
        iter,
        time::{Duration, Instant},
    },
    tui::{buffer::Buffer, layout::Rect, style::Style, widgets::StatefulWidget},
};

pub struct Spinner {
    style: Style,
    interval: Duration,
}

impl Spinner {
    builder_method! {
        pub style: Style
    }

    pub fn per_rotation(self, duration: Duration) -> Self {
        Self {
            interval: duration / SYMBOLS.len() as u32,
            ..self
        }
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Spinner {
            style: <_>::default(),
            interval: <_>::default(),
        }
        .per_rotation(Duration::from_millis(500))
    }
}

type Symbol = ([usize; 2], char);
type Symbols = iter::Copied<std::slice::Iter<'static, Symbol>>;

type Iter = iter::Cycle<
    iter::Zip<Symbols, iter::Chain<iter::Skip<Symbols>, std::option::IntoIter<Symbol>>>,
>;

pub struct SpinnerState {
    symbols: Iter,
    last_symbols: (Symbol, Symbol),
    last_step: Instant,
}

impl SpinnerState {
    fn steps(&mut self, interval: Duration) -> usize {
        let steps = self.last_step.elapsed().as_millis() / interval.as_millis();

        if steps > 0 {
            self.last_step = Instant::now()
        }

        steps as _
    }
}

impl Default for SpinnerState {
    fn default() -> Self {
        let mut symbols = symbol_pairs();
        let last_symbols = symbols.next().unwrap();

        SpinnerState {
            symbols,
            last_symbols,
            last_step: Instant::now(),
        }
    }
}

const SYMBOLS: &[Symbol] = &[
    ([1, 0], '─'),
    ([2, 0], '╮'),
    ([2, 1], '╯'),
    ([1, 1], '─'),
    ([0, 1], '╰'),
    ([0, 0], '╭'),
];

fn symbol_pairs() -> Iter {
    symbols()
        .zip(symbols().skip(1).chain(symbols().next()))
        .cycle()
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
        if area.width < WIDTH || area.height < HEIGHT {
            return
        }

        let mut cells = [[' '; WIDTH as _]; HEIGHT as _];

        let steps = state.steps(self.interval);

        let (a, b) = match steps {
            0 => state.last_symbols,
            n => state.symbols.nth(n - 1).unwrap(),
        };

        state.last_symbols = (a, b);

        insert(a, &mut cells);
        insert(b, &mut cells);

        for (y, row) in cells.iter().enumerate() {
            for (x, sym) in row.iter().enumerate() {
                buf.get_mut(area.x + x as u16, area.y + y as u16)
                    .set_char(*sym)
                    .set_style(self.style);
            }
        }
    }
}

fn insert(([x, y], sym): Symbol, cells: &mut Cells) {
    cells[y][x] = sym
}
