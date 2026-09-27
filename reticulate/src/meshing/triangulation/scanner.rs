use crate::{
    Coords2D,
    axis::Axis,
    meshing::{
        bitfields::{Claims, PLANE_SIDE, Parity},
        triangulation::{WeavedTriangle, stepper::Stepper},
    },
    utils::bitmasks,
};

pub struct Scanner<'a> {
    parity: &'a Parity<'a>,
    axis: Axis,
    axis_u: Axis,
    axis_coord: usize,
    edges: &'a [u32],
    prev_edges: Option<&'a [u32]>,
}

impl<'a> Scanner<'a> {
    pub fn new(parity: &'a Parity<'a>, axis: Axis, axis_coord: usize) -> Self {
        let axis_u = axis.next();
        let axis_v = axis_u.next();
        Scanner {
            parity,
            axis,
            axis_u,
            axis_coord,
            edges: parity.gather_edges(axis_v, axis_coord),
            prev_edges: (axis_coord > 0).then(|| parity.gather_edges(axis_v, axis_coord - 1)),
        }
    }

    pub fn scan_seed(
        &self,
        claims: &Claims,
        seed: Coords2D,
        mut emit: impl FnMut(WeavedTriangle),
    ) -> [u32; PLANE_SIDE] {
        let claimed_row = |row: usize| {
            if row < PLANE_SIDE {
                claims.claimed_row(self.axis, self.axis_coord, row)
            } else {
                0
            }
        };

        let seed_row = seed.u as usize;
        let mut stepper =
            Stepper::from_bottom_edge(self.edge(seed_row, claimed_row(seed_row)), seed.v, seed.u);

        for row in seed_row + 1..=PLANE_SIDE {
            if stepper.runs.count() == 0 {
                break;
            }
            let claimed = claimed_row(row);
            let live = stepper.runs.fill_mask();
            stepper.step_to(
                row,
                self.edge(row, claimed),
                self.guard(row, live, claimed),
                &mut emit,
            );
        }

        stepper.filled
    }

    pub fn debug_assert_within_footprint(&self, filled: &[u32; PLANE_SIDE]) {
        #[cfg(debug_assertions)]
        for (u, &claimed) in filled.iter().enumerate() {
            let footprint = self.parity.gather_footprint(self.axis, self.axis_coord, u);
            debug_assert!(
                claimed & !footprint == 0,
                "strip claimed non-face cells on {:?}@{}, row {u}: {:032b}",
                self.axis,
                self.axis_coord,
                claimed & !footprint
            );
        }
    }

    /// Walls at `row`: both slabs plus the boundaries of claimed cells.
    fn edge(&self, row: usize, claimed: u32) -> u32 {
        if row >= PLANE_SIDE {
            return 0;
        }
        self.edges[row]
            | self.prev_edges.map_or(0, |p| p[row])
            | bitmasks::segment_boundaries(claimed)
    }

    /// Which of the `live` columns cannot continue into `row`. Masked by
    /// `live`, so no bit can ever create a phantom run. Outside the chunk
    /// everything is considered empty, so every live column is blocked.
    fn guard(&self, row: usize, live: u32, claimed: u32) -> u32 {
        if row >= PLANE_SIDE {
            return live;
        }
        let start = live.trailing_zeros() as usize;
        let end = (32 - live.leading_zeros()) as usize;
        let mut g = self
            .parity
            .gather_guard(self.axis_u, self.axis_coord, row, start, end);
        if self.axis_coord > 0 {
            g |= self
                .parity
                .gather_guard(self.axis_u, self.axis_coord - 1, row, start, end);
        }
        (g | claimed) & live
    }
}
