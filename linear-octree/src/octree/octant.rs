#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Octant(u8);

impl Octant {
    #[inline(always)]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn from_u8(value: u8) -> Self {
        Octant(value)
    }
}

impl From<u8> for Octant {
    fn from(value: u8) -> Self {
        Octant(value)
    }
}

impl From<Octant> for u8 {
    fn from(value: Octant) -> Self {
        value.0
    }
}
