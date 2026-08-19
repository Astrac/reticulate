use std::ops::Sub;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Depth(pub u8);

impl From<u8> for Depth {
    fn from(value: u8) -> Self {
        Depth(value)
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
