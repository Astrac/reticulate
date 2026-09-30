use crate::{
    meshing::{
        bitfields::{PLANE_SIDE, Parity},
        surface_layer::SurfaceLayer,
        triangulation::{Triangle2D, edges::Edges, stepper::Stepper},
    },
    utils::bitmasks,
};

pub struct Weaver<'a> {
    parity: &'a Parity<'a>,
}

impl<'a> Weaver<'a> {
    pub fn new(parity: &'a Parity<'a>) -> Self {
        Weaver { parity }
    }

    pub fn weave_region(
        &self,
        layer: SurfaceLayer,
        region: &[u32; PLANE_SIDE],
        mut emit: impl FnMut(Triangle2D),
    ) {
        let edges = Edges::from_parity(&self.parity, layer);
        self.weave(layer, &edges, &region, &mut emit);
    }

    fn weave(
        &self,
        layer: SurfaceLayer,
        edges: &Edges,
        region: &[u32; PLANE_SIDE],
        emit: &mut impl FnMut(Triangle2D),
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
            stepper.weave_to(row, cells, extra, |t| emit(t));
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
        points: u32,
    ) -> u32 {
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
        let cells = points | (points >> 1);
        let across = if line < PLANE_SIDE {
            self.parity.boundary_guard(layer, line, cells)
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
