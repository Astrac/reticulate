use crate::{
    Coords2D,
    meshing::{
        bitfields::{Claims, PLANE_SIDE, Parity},
        surface_layer::SurfaceLayer,
        triangulation::{edges::Edges, runs::Runs},
    },
    utils::bitmasks,
};

// TODO - The division between this and Weaver is a bit messy, reorganize
pub struct Discoverer<'a> {
    // edges: &'a [u32],
    parity: &'a Parity<'a>,
    // prev_edges: Option<&'a [u32]>,
}

impl<'a> Discoverer<'a> {
    pub fn new(parity: &'a Parity<'a>) -> Self {
        // let columns_axis = layer.axis.next();
        // let rows_axis = columns_axis.next();
        Discoverer {
            parity,
            // edges: parity.gather_edges(SurfaceLayer::new(rows_axis, layer.layer)),
            // prev_edges: (layer.layer > 0)
            //     .then(|| parity.gather_edges(SurfaceLayer::new(rows_axis, layer.layer - 1))),
        }
    }

    pub fn discover(
        &self,
        layer: SurfaceLayer,
        claims: &Claims,
        edges: &Edges,
        seed: Coords2D,
    ) -> [u32; PLANE_SIDE] {
        let mut region = [0u32; PLANE_SIDE];
        let seed_row = seed.u as usize; // row: u, column: v
        region[seed_row] = self.expand(layer, claims, edges, seed_row, 1 << seed.v);

        loop {
            let mut changed = false;
            for to in 1..PLANE_SIDE {
                changed |= self.grow(layer, claims, edges, &mut region, to - 1, to);
            }
            for to in (0..PLANE_SIDE - 1).rev() {
                changed |= self.grow(layer, claims, edges, &mut region, to + 1, to);
            }
            if !changed {
                return region;
            }
        }
    }

    fn grow(
        &self,
        layer: SurfaceLayer,
        claims: &Claims,
        edges: &Edges,
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
        let columns = source
            & !self.parity.boundary_guard(layer, boundary, source)
            & !claims.claimed_row(layer, to);
        if columns == 0 {
            return false;
        }
        region[to] |= self.expand(layer, claims, edges, to, columns);
        true
    }

    fn expand(
        &self,
        layer: SurfaceLayer,
        claims: &Claims,
        edges: &Edges,
        row: usize,
        columns: u32,
    ) -> u32 {
        let walls = edges.walls(row) | bitmasks::segment_boundaries(claims.claimed_row(layer, row));
        Runs::from_blocks(walls, columns).fill_mask()
    }
}
