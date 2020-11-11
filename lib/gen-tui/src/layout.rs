pub trait RectExt: Sized + Copy {
    fn shrink_left(self, amount: u16) -> Self;
    fn shrink_right(self, amount: u16) -> Self;
    fn shrink_top(self, amount: u16) -> Self;
    fn shrink_bottom(self, amount: u16) -> Self;

    #[inline]
    fn shrink(self, margin: Margin) -> Self {
        self.shrink_left(margin.left)
            .shrink_right(margin.right)
            .shrink_top(margin.top)
            .shrink_bottom(margin.bottom)
    }
    
    fn grow_left(self, amount: u16) -> Self;
    fn grow_right(self, amount: u16) -> Self;
    fn grow_top(self, amount: u16) -> Self;
    fn grow_bottom(self, amount: u16) -> Self;

    #[inline]
    fn grow(self, margin: Margin) -> Self {
        self.grow_left(margin.left)
            .grow_right(margin.right)
            .grow_top(margin.top)
            .grow_bottom(margin.bottom)
    }

    fn from_left(self, width: u16) -> Self;
    fn from_right(self, width: u16) -> Self;
    fn from_top(self, height: u16) -> Self;
    fn from_bottom(self, height: u16) -> Self;

    #[inline]
    fn split_x(self, x: u16) -> (Self, Self) {
        (self.from_left(x), self.shrink_left(x))
    }

    #[inline]
    fn split_y(self, y: u16) -> (Self, Self) {
        (self.from_top(y), self.shrink_top(y))
    }

    fn split_ratio_x(self, ratio: f32) -> (Self, Self);
    fn split_ratio_y(self, ratio: f32) -> (Self, Self);

    fn contains(self, x: u16, y: u16) -> bool;
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

    #[inline]
    fn grow_left(self, amount: u16) -> Self {
        let (x, width) = grow_adjust(self.x, self.width, amount);

        Self { x, width, ..self }
    }

    #[inline]
    fn grow_right(self, amount: u16) -> Self {
        Self {
            width: self.width.saturating_add(amount),
            ..self
        }
    }

    #[inline]
    fn grow_top(self, amount: u16) -> Self {
        let (y, height) = grow_adjust(self.y, self.height, amount);

        Self { y, height, ..self }
    }
    
    #[inline]
    fn grow_bottom(self, amount: u16) -> Self {
        Self {
            height: self.height.saturating_add(amount),
            ..self
        }
    }

    #[inline]
    fn from_left(self, width: u16) -> Self {
        self.shrink_right(self.width.saturating_sub(width))
    }

    #[inline]
    fn from_right(self, width: u16) -> Self {
        self.shrink_left(self.width.saturating_sub(width))
    }

    #[inline]
    fn from_top(self, height: u16) -> Self {
        self.shrink_bottom(self.height.saturating_sub(height))
    }

    #[inline]
    fn from_bottom(self, height: u16) -> Self {
        self.shrink_top(self.height.saturating_sub(height))
    }

    #[inline]
    fn split_ratio_x(self, ratio: f32) -> (Self, Self) {
        let x = f32::from(self.width) * ratio;

        self.split_x(x.round() as u16)
    }

    #[inline]
    fn split_ratio_y(self, ratio: f32) -> (Self, Self) {
        let y = f32::from(self.height) * ratio;

        self.split_y(y.round() as u16)
    }

    #[inline]
    fn contains(self, x: u16, y: u16) -> bool {
        x >= self.left()
            && x < self.right()
            && y >= self.top()
            && y < self.bottom()
    }
}

fn shrink_adjust(coord: u16, size: u16, amount: u16) -> (u16, u16) {
    (
        coord.saturating_add(amount),
        size.saturating_sub(amount)
    )
}

fn grow_adjust(coord: u16, size: u16, amount: u16) -> (u16, u16) {
    (
        coord.saturating_sub(amount),
        size.saturating_add(amount)
    )
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Margin {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16
}

macro_rules! dirs {
    ($($dir:ident),+) => {
        $(
            pub fn $dir($dir: u16) -> Margin {
                Margin { $dir, ..<_>::default() }
            }
        )+
    };
}

impl Margin {
    dirs!(left, right, top, bottom);
}
