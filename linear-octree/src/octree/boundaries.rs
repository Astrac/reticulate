use crate::face::Face;

// TODO: Remove this file, it seems useless

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boundary(pub(crate) u8);

impl Boundary {
    #[inline(always)]
    pub const fn boundary_mask(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn as_index(&self) -> usize {
        let mask = self.0 as usize;
        (3 * (mask >> 3)) + ((mask & 0b111) >> 1)
    }

    #[inline(always)]
    pub fn as_face_positive(&self) -> Face {
        Face::from(self.0 & 0b111)
    }

    #[inline(always)]
    pub const fn is_inner(&self) -> bool {
        self.0 & 0b10000 != 0
    }
}

impl From<u8> for Boundary {
    fn from(value: u8) -> Self {
        Boundary(value)
    }
}

impl From<Boundary> for u8 {
    fn from(value: Boundary) -> Self {
        value.0
    }
}

#[allow(dead_code)]
pub const OUTER_NEGATIVE_X: Boundary = Boundary(0b00001);
#[allow(dead_code)]
pub const OUTER_NEGATIVE_Y: Boundary = Boundary(0b00010);
#[allow(dead_code)]
pub const OUTER_NEGATIVE_Z: Boundary = Boundary(0b00100);
#[allow(dead_code)]
pub const OUTER_POSITIVE_X: Boundary = Boundary(0b01001);
#[allow(dead_code)]
pub const OUTER_POSITIVE_Y: Boundary = Boundary(0b01010);
#[allow(dead_code)]
pub const OUTER_POSITIVE_Z: Boundary = Boundary(0b01100);
#[allow(dead_code)]
pub const INNER_X: Boundary = Boundary(0b10001);
#[allow(dead_code)]
pub const INNER_Y: Boundary = Boundary(0b10010);
#[allow(dead_code)]
pub const INNER_Z: Boundary = Boundary(0b10100);

#[allow(dead_code)]
pub const INNERS: [Boundary; 3] = [INNER_X, INNER_Y, INNER_Z];
#[allow(dead_code)]
pub const OUTERS: [Boundary; 6] = [
    OUTER_NEGATIVE_X,
    OUTER_NEGATIVE_Y,
    OUTER_NEGATIVE_Z,
    OUTER_POSITIVE_X,
    OUTER_POSITIVE_Y,
    OUTER_POSITIVE_Z,
];

#[allow(dead_code)]
pub const BOUNDARIES: [Boundary; 9] = {
    let mut list = [Boundary(0); 9];

    let mut i = 0;
    while i < INNERS.len() {
        list[INNERS[i].as_index()] = INNERS[i];
        i += 1;
    }

    let mut i = 0;
    while i < OUTERS.len() {
        list[OUTERS[i].as_index()] = OUTERS[i];
        i += 1;
    }

    list
};
