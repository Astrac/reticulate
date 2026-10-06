use crate::{BoundaryMask, CellMask};

#[derive(Clone, Copy, Debug, Default)]
pub struct RunMasks {
    fill: CellMask,
    scan: BoundaryMask,
}

impl RunMasks {
    #[inline(always)]
    pub const fn new(fill: CellMask, scan: BoundaryMask) -> RunMasks {
        RunMasks { fill, scan }
    }

    #[inline(always)]
    pub const fn fill(&self) -> CellMask {
        self.fill
    }

    #[inline(always)]
    pub const fn scan(&self) -> BoundaryMask {
        self.scan
    }
}
