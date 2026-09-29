use crate::{
    Coords2D,
    axis::Axis,
    meshing::{
        bitfields::{Claims, PLANE_SIDE, Parity},
        triangulation::runs::Runs,
    },
    utils::bitmasks,
};

// TODO - The division between this and Scanner is a bit messy, reorganize
pub struct Discoverer<'a> {
    axis: Axis,
    axis_coord: usize,
    columns_axis: Axis,
    edges: &'a [u32],
    parity: &'a Parity<'a>,
    prev_edges: Option<&'a [u32]>,
}

impl<'a> Discoverer<'a> {
    pub fn new(parity: &'a Parity<'a>, axis: Axis, axis_coord: usize) -> Self {
        let columns_axis = axis.next();
        let rows_axis = columns_axis.next();
        Discoverer {
            parity,
            axis,
            columns_axis,
            axis_coord,
            edges: parity.gather_edges(rows_axis, axis_coord),
            prev_edges: (axis_coord > 0).then(|| parity.gather_edges(rows_axis, axis_coord - 1)),
        }
    }

    pub fn discover(&self, claims: &Claims, seed: Coords2D) -> [u32; PLANE_SIDE] {
        let mut region = [0u32; PLANE_SIDE];
        let seed_row = seed.u as usize; // row: u, column: v
        region[seed_row] = self.expand(claims, seed_row, 1 << seed.v);

        loop {
            let mut changed = false;
            for to in 1..PLANE_SIDE {
                changed |= self.grow(claims, &mut region, to - 1, to);
            }
            for to in (0..PLANE_SIDE - 1).rev() {
                changed |= self.grow(claims, &mut region, to + 1, to);
            }
            if !changed {
                return region;
            }
        }
    }

    fn grow(
        &self,
        claims: &Claims,
        region: &mut [u32; PLANE_SIDE],
        from: usize,
        to: usize,
    ) -> bool {
        let source = region[from] & !region[to];
        if source == 0 {
            return false;
        }
        // Parity bit b is the transition between rows b - 1 and b.
        let boundary = from.max(to);
        let columns =
            source & !self.boundary_guard(boundary, source) & !self.claimed_row(claims, to);
        if columns == 0 {
            return false;
        }
        region[to] |= self.expand(claims, to, columns);
        true
    }

    fn expand(&self, claims: &Claims, row: usize, columns: u32) -> u32 {
        let walls = self.walls(row) | bitmasks::segment_boundaries(self.claimed_row(claims, row));
        Runs::from_blocks(walls, columns).fill_mask()
    }

    /// Walls of `row` in both slabs.
    pub fn walls(&self, row: usize) -> u32 {
        self.edges[row] | self.prev_edges.map_or(0, |p| p[row])
    }

    fn claimed_row(&self, claims: &Claims, row: usize) -> u32 {
        claims.claimed_row(self.axis, self.axis_coord, row)
    }

    /// Columns of `mask` with a transition across parity `boundary` in
    /// either slab.
    pub fn boundary_guard(&self, boundary: usize, mask: u32) -> u32 {
        let start = mask.trailing_zeros() as usize;
        let end = (32 - mask.leading_zeros()) as usize;
        let mut g =
            self.parity
                .gather_guard(self.columns_axis, self.axis_coord, boundary, start, end);
        if self.axis_coord > 0 {
            g |= self.parity.gather_guard(
                self.columns_axis,
                self.axis_coord - 1,
                boundary,
                start,
                end,
            );
        }
        g & mask
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
