use crate::meshing::triangulation::run::Run;

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
    pub const fn from_runs(runs: &[Run; 16], count: usize) -> RunMasks {
        let mut fill = 0;
        let mut scan = 0;
        let mut index = 0;

        while index < count {
            fill |= runs[index].fill_mask();
            scan |= runs[index].scan_mask();
            index += 1;
        }

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
