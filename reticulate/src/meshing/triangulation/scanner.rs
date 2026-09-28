use crate::{
    Coords2D,
    axis::Axis,
    meshing::{
        bitfields::{Claims, PLANE_SIDE, Parity},
        triangulation::{Triangle2D, direction::Direction, stepper::Stepper},
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
    // `new` and `debug_assert_within_footprint` unchanged.
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
        mut emit: impl FnMut(Triangle2D),
    ) -> [u32; PLANE_SIDE] {
        let mut filled = [0u32; PLANE_SIDE];
        let s = seed.u as usize;

        let up = Stepper::from_live(self.edge(claims, Direction::Up, s), 1 << seed.v, s as u32);
        let seed_fill = up.runs.fill_mask();
        self.sweep(claims, Direction::Up, up, s, &mut emit, &mut filled);

        // TODO - this is unnecessary once the valley discovery is properly implemented
        if s > 0 {
            let r0 = PLANE_SIDE - s;
            let live = seed_fill & !self.guard(claims, Direction::Down, r0, seed_fill);
            if live != 0 {
                let edge = self.edge(claims, Direction::Down, r0);
                let down = Stepper::from_live(edge, live, r0 as u32);
                self.sweep(claims, Direction::Down, down, r0, &mut emit, &mut filled);
            }
        }

        filled
    }

    fn sweep(
        &self,
        claims: &Claims,
        dir: Direction,
        mut stepper: Stepper,
        start: usize,
        emit: &mut impl FnMut(Triangle2D),
        filled: &mut [u32; PLANE_SIDE],
    ) {
        for r in start + 1..=PLANE_SIDE {
            if stepper.runs.count() == 0 {
                break;
            }
            let live = stepper.runs.fill_mask();
            stepper.step_to(
                r,
                self.edge(claims, dir, r),
                self.guard(claims, dir, r, live),
                |t| emit(dir.map(t)),
            );
        }
        for r in 0..PLANE_SIDE {
            if let Some(row) = dir.plane_row(r) {
                filled[row] |= stepper.filled[r];
            }
        }
    }

    fn claimed(&self, claims: &Claims, row: usize) -> u32 {
        claims.claimed_row(self.axis, self.axis_coord, row)
    }

    fn edge(&self, claims: &Claims, dir: Direction, r: usize) -> u32 {
        match dir.plane_row(r) {
            None => 0,
            Some(row) => {
                self.edges[row]
                    | self.prev_edges.map_or(0, |p| p[row])
                    | bitmasks::segment_boundaries(self.claimed(claims, row))
            }
        }
    }

    fn guard(&self, claims: &Claims, dir: Direction, r: usize, live: u32) -> u32 {
        let Some(row) = dir.plane_row(r) else {
            return live;
        };
        let boundary = dir.boundary(r);
        let start = live.trailing_zeros() as usize;
        let end = (32 - live.leading_zeros()) as usize;
        let mut g = self
            .parity
            .gather_guard(self.axis_u, self.axis_coord, boundary, start, end);
        if self.axis_coord > 0 {
            g |= self
                .parity
                .gather_guard(self.axis_u, self.axis_coord - 1, boundary, start, end);
        }
        (g | self.claimed(claims, row)) & live
    }

    #[cfg(debug_assertions)]
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
}
