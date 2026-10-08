use crate::{
    BoundaryMask, CellMask, PLANE_SIDE,
    meshing::{
        bitfields::Parity,
        surface_layer::SurfaceLayer,
        triangulation::{Triangle2D, edges::Edges, stepper::Stepper},
    },
    utils::bitmasks,
};

pub struct Weaver<'a> {
    parity: &'a Parity<'a>,
}

impl<'a> Weaver<'a> {
    #[inline(always)]
    pub fn new(parity: &'a Parity<'a>) -> Self {
        Weaver { parity }
    }

    #[inline(always)]
    pub fn weave_region(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        region: &[u32; PLANE_SIDE],
        mut emit: impl FnMut(Triangle2D),
    ) {
        let Some(first) = region.iter().position(|&cells| cells != 0) else {
            return;
        };
        let last = PLANE_SIDE - 1 - region.iter().rev().position(|&cells| cells != 0).unwrap();

        let edge = bitmasks::segment_boundaries(region[first]);
        let mut stepper = Stepper::from_live(edge, region[first], first as u32);
        let start_extra = self.partition_corners(
            layer,
            edges,
            first,
            bitmasks::boundary_points(0, region[first]),
        );
        stepper.add_vertices(start_extra, first as u32);

        for row in first + 1..=last + 1 {
            let cells = if row < PLANE_SIDE { region[row] } else { 0 };
            let extra = self.partition_corners(
                layer,
                edges,
                row,
                bitmasks::boundary_points(region[row - 1], cells),
            );
            stepper.weave_to(row, cells, extra, &mut emit);
        }

        debug_assert_eq!(
            stepper.filled, *region,
            "weaving must cover exactly the region"
        );
    }

    fn partition_corners(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        line: usize,
        points: BoundaryMask,
    ) -> BoundaryMask {
        if points == 0 {
            return 0;
        }
        let up = if line < PLANE_SIDE {
            edges.walls(line)
        } else {
            0
        };
        let down = if line > 0 { edges.walls(line - 1) } else { 0 };
        // Each point c needs the segments under cells c - 1 and c.
        let cells = (points | (points >> 1)) as CellMask;
        let across = if line < PLANE_SIDE {
            self.parity.boundary_guard(layer, line, cells) as BoundaryMask
        } else {
            0
        };
        let (left, right) = (across << 1, across);

        let any = up | down | left | right;
        let vertical = up & down & !left & !right;
        let horizontal = left & right & !up & !down;
        any & !vertical & !horizontal & points
    }
}
