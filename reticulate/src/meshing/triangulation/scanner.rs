use crate::{
    Coords2D,
    meshing::{
        bitfields::{Claims, PLANE_SIDE},
        triangulation::{Triangle2D, discoverer::Discoverer, stepper::Stepper},
    },
    utils::bitmasks,
};

pub struct Scanner<'a> {
    pub discoverer: Discoverer<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(discoverer: Discoverer<'a>) -> Self {
        Scanner { discoverer }
    }

    pub fn scan_seed(
        &self,
        claims: &Claims,
        seed: Coords2D,
        mut emit: impl FnMut(Triangle2D),
    ) -> [u32; PLANE_SIDE] {
        let region = self.discoverer.discover(claims, seed);
        self.weave(&region, &mut emit);
        region
    }

    fn weave(&self, region: &[u32; PLANE_SIDE], emit: &mut impl FnMut(Triangle2D)) {
        let Some(first) = region.iter().position(|&cells| cells != 0) else {
            return;
        };
        let last = PLANE_SIDE - 1 - region.iter().rev().position(|&cells| cells != 0).unwrap();

        let edge = bitmasks::segment_boundaries(region[first]);
        let mut stepper = Stepper::from_live(edge, region[first], first as u32);
        for row in first + 1..=last + 1 {
            let cells = if row < PLANE_SIDE { region[row] } else { 0 };
            stepper.weave_to(row, cells, |t| emit(t));
        }

        debug_assert_eq!(
            stepper.filled, *region,
            "weaving must cover exactly the region"
        );
    }
}
