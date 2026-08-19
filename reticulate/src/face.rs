use crate::axis::Axis;

#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Face(pub u8);

impl Face {
    #[inline(always)]
    pub const fn face_mask(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn is_positive(&self) -> bool {
        self.0 & 0b1000 == 0
    }

    #[inline(always)]
    pub const fn as_index(&self) -> usize {
        (self.0.trailing_zeros() as usize) + (3 * ((self.0 as usize) >> 3))
    }

    pub fn axis(&self) -> Axis {
        Axis(self.0 & 0b111)
    }
}

impl From<u8> for Face {
    fn from(value: u8) -> Self {
        Face(value)
    }
}

impl From<Face> for u8 {
    fn from(value: Face) -> Self {
        value.0
    }
}

pub const FACE_X_P: Face = Face(0b0001);
pub const FACE_Y_P: Face = Face(0b0010);
pub const FACE_Z_P: Face = Face(0b0100);
pub const FACE_X_N: Face = Face(0b1001);
pub const FACE_Y_N: Face = Face(0b1010);
pub const FACE_Z_N: Face = Face(0b1100);
