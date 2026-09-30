use std::fmt::Debug;

use crate::axis::Axis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Negative(Axis),
    Positive(Axis),
}

impl Face {
    #[inline(always)]
    pub const fn axis(&self) -> Axis {
        match *self {
            Face::Negative(axis) => axis,
            Face::Positive(axis) => axis,
        }
    }

    #[inline(always)]
    pub const fn is_positive(&self) -> bool {
        matches!(*self, Face::Negative(_))
    }

    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        match self {
            Face::Negative(axis) => axis.as_usize() + 3,
            Face::Positive(axis) => axis.as_usize(),
        }
    }
}
