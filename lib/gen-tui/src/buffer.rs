use {
    crate::layout::RectExt,
    tui::{
        buffer::{Buffer, Cell},
        layout::Rect,
    },
};

pub trait BufferExt: Sized {
    fn copy_from(&mut self, area: Rect, other: Self, other_area: Rect);
    fn try_get(&self, x: u16, y: u16) -> Option<&Cell>;
    fn try_get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell>;
}

impl BufferExt for Buffer {
    fn copy_from(&mut self, area: Rect, mut other: Self, other_area: Rect) {
        let width = area.width.min(other_area.width);
        let height = area.height.min(other_area.height);
        let other_area = Rect {
            width,
            height,
            ..other_area
        };

        cells_mut(self, area)
            .zip(cells_mut(&mut other, other_area))
            .for_each(|(a, b)| std::mem::swap(a, b))
    }

    fn try_get(&self, x: u16, y: u16) -> Option<&Cell> {
        if self.area.contains(x, y) {
            self.get(x, y).into()
        } else {
            None
        }
    }

    fn try_get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if self.area.contains(x, y) {
            self.get_mut(x, y).into()
        } else {
            None
        }
    }
}

fn cells_mut(buf: &mut Buffer, area: Rect) -> impl Iterator<Item = &mut Cell> + '_ {
    let x = area.x.saturating_sub(buf.area.x) as _;
    let y = area.y.saturating_sub(buf.area.y) as _;

    buf.content
        .chunks_exact_mut(buf.area.width as _)
        .skip(y)
        .flat_map(move |row| row.iter_mut().skip(x).take(area.width as _))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn copy() {
        let a_area = Rect {
            x: 20,
            y: 20,
            width: 5,
            height: 10,
        };
        let a = Buffer::filled(a_area, &Cell {
            symbol: "ö".into(),
            ..<_>::default()
        });

        let mut b = Buffer::empty(Rect {
            x: 3,
            y: 7,
            width: 10,
            height: 5,
        });
        b.copy_from(b.area, a, a_area);

        for x in b.area.x..b.area.x + 5 {
            for y in b.area.top()..b.area.bottom() {
                assert_eq!(b.get(x, y).symbol, "ö", "pos = ({}/{})", x, y)
            }
        }
    }
}
