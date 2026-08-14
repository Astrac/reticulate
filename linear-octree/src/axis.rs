use crate::face::Face;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis(u8);

impl Axis {
    #[inline(always)]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn as_index(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline(always)]
    pub const fn as_positive_face(&self) -> Face {
        Face(self.0)
    }

    #[inline(always)]
    pub const fn as_negative_face(&self) -> Face {
        Face(self.0 | 0b1000)
    }
}

impl From<u8> for Axis {
    fn from(value: u8) -> Self {
        Axis(value)
    }
}

impl From<Axis> for u8 {
    fn from(value: Axis) -> Self {
        value.0
    }
}

pub const AXIS_X: Axis = Axis(0b00001);
pub const AXIS_Y: Axis = Axis(0b00010);
pub const AXIS_Z: Axis = Axis(0b00100);

pub const AXES: [Axis; 3] = [AXIS_X, AXIS_Y, AXIS_Z];
