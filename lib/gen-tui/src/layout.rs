pub trait RectExt: Sized {
    fn shrink_left(self, amount: u16) -> Self;
    fn shrink_right(self, amount: u16) -> Self;
    fn shrink_top(self, amount: u16) -> Self;
    fn shrink_bottom(self, amount: u16) -> Self;
}

impl RectExt for tui::layout::Rect {
    #[inline]
    fn shrink_left(self, amount: u16) -> Self {
        let (x, width) = shrink_adjust(self.x, self.width, amount);

        Self { x, width, ..self }
    }

    #[inline]
    fn shrink_right(self, amount: u16) -> Self {
        Self {
            width: self.width.saturating_sub(amount),
            ..self
        }
    }

    #[inline]
    fn shrink_top(self, amount: u16) -> Self {
        let (y, height) = shrink_adjust(self.y, self.height, amount);

        Self { y, height, ..self }
    }

    #[inline]
    fn shrink_bottom(self, amount: u16) -> Self {
        Self {
            height: self.height.saturating_sub(amount),
            ..self
        }
    }
}

fn shrink_adjust(coord: u16, size: u16, amount: u16) -> (u16, u16) {
    (
        coord.saturating_add(amount),
        size.saturating_sub(amount)
    )
}
