use std::fmt::Debug;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

pub const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

impl Axis {
    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    pub const fn next(&self) -> Axis {
        AXES[(self.as_usize() + 1) % 3]
    }
}
