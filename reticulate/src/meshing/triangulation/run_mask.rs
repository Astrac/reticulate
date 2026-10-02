#[derive(Clone, Copy, Debug, Default)]
pub struct RunMasks {
    fill: u32,
    scan: u32,
}

impl RunMasks {
    #[inline(always)]
    pub const fn new(fill: u32, scan: u32) -> RunMasks {
        RunMasks { fill, scan }
    }

    #[inline(always)]
    pub const fn fill(&self) -> u32 {
        self.fill
    }

    #[inline(always)]
    pub const fn scan(&self) -> u32 {
        self.scan
    }
}
