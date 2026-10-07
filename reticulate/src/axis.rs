use std::fmt::Debug;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    pub const fn next(&self) -> Axis {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::Z,
            Axis::Z => Axis::X,
        }
    }

    #[inline(always)]
    pub const fn prev(&self) -> Axis {
        match self {
            Axis::X => Axis::Z,
            Axis::Y => Axis::X,
            Axis::Z => Axis::Y,
        }
    }
}
