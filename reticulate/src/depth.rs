use std::ops::{Add, Sub};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Depth(pub u8);

impl Depth {
    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Add<u8> for Depth {
    type Output = Depth;

    fn add(self, rhs: u8) -> Self::Output {
        Depth(self.0 + rhs)
    }
}

impl Sub<u8> for Depth {
    type Output = Depth;

    fn sub(self, rhs: u8) -> Self::Output {
        Depth(self.0 - rhs)
    }
}

impl From<u8> for Depth {
    fn from(value: u8) -> Self {
        Depth(value)
    }
}

impl From<usize> for Depth {
    fn from(value: usize) -> Self {
        Depth(value as u8)
    }
}

impl From<Depth> for u8 {
    fn from(value: Depth) -> Self {
        value.0
    }
}

impl Sub<Depth> for u8 {
    type Output = u8;

    fn sub(self, rhs: Depth) -> Self::Output {
        self - rhs.0
    }
}
