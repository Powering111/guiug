#[derive(Clone, Copy, Debug)]
pub(crate) struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn dimension(self) -> Dimension {
        Dimension {
            width: self.w,
            height: self.h,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Dimension {
    pub width: i32,
    pub height: i32,
}

impl Dimension {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}
