use crate::meshing::triangulation::{WeavedTriangle, stepper::INACTIVE_VERTEX};

#[derive(Clone, Debug, Default)]
pub struct Zipper {
    pub bottom_vs: [u8; 32],
    pub bottom_u_mask: u32,
}

impl Zipper {
    pub fn zip(
        &mut self,
        run_mask: u32,
        top_row: usize,
        top_u_mask: u32,
        on_triangle: &mut impl FnMut(WeavedTriangle),
    ) {
        let mut triangle = WeavedTriangle::default();
        let mut bottom_u_mask = self.bottom_u_mask & run_mask;
        let mut top_u_mask = top_u_mask & run_mask;

        let (Some(mut top_u), Some(mut bottom_u)) =
            (top_u_mask.lowest_one(), bottom_u_mask.lowest_one())
        else {
            return;
        };

        top_u_mask -= top_u_mask.isolate_lowest_one();
        bottom_u_mask -= bottom_u_mask.isolate_lowest_one();

        loop {
            triangle.vertices[0] = (top_u as u8, top_row as u8);
            triangle.vertices[1] = (bottom_u as u8, self.bottom_vs[bottom_u as usize]);

            let next_top = top_u_mask
                .lowest_one()
                .map(|v| v as u8)
                .unwrap_or(INACTIVE_VERTEX);
            let next_bottom = bottom_u_mask
                .lowest_one()
                .map(|v| v as u8)
                .unwrap_or(INACTIVE_VERTEX);

            // A cliff is when the next bottom vertex is below the current one
            let is_bottom_cliff = next_bottom != INACTIVE_VERTEX
                && self.bottom_vs[next_bottom as usize] < self.bottom_vs[bottom_u as usize];

            if next_bottom == INACTIVE_VERTEX && next_top == INACTIVE_VERTEX {
                // No vertices remaining
                break;
            } else if next_top != INACTIVE_VERTEX && (next_top < next_bottom || is_bottom_cliff) {
                // Choose next_top if it is closer or if next bottom is a cliff
                top_u_mask -= top_u_mask.isolate_lowest_one();
                top_u = next_top as u32;

                // Emit triangle
                triangle.vertices[2] = (top_u as u8, top_row as u8);
                on_triangle(triangle);
            } else if is_bottom_cliff && bottom_u > top_u {
                // TODO - This is a new valley, it should trigger discovery
                break;
            } else if !is_bottom_cliff && (next_bottom as u32) < top_u {
                // Skip bottom vertices that are not needed on this run but don't remove
                // them from present/vertices
                bottom_u_mask -= bottom_u_mask.isolate_lowest_one();
                bottom_u = next_bottom as u32;
            } else {
                // Otherwise choose next_bottom
                bottom_u_mask -= bottom_u_mask.isolate_lowest_one();
                bottom_u = next_bottom as u32;

                // Remove this vertex
                self.bottom_u_mask &= !(1 << bottom_u);

                // Emit triangle
                triangle.vertices[2] = (bottom_u as u8, self.bottom_vs[bottom_u as usize]);
                on_triangle(triangle);
            }
        }

        self.bottom_u_mask |= 1 << bottom_u;
    }

    #[inline(always)]
    pub const fn advance(&mut self, row: usize, mut mask: u32, scan_mask: u32) {
        self.bottom_u_mask = (self.bottom_u_mask & !mask) | (mask & scan_mask);
        while let Some(x) = mask.lowest_one() {
            mask &= !mask.isolate_lowest_one();
            self.bottom_vs[x as usize] = row as u8;
        }
    }
}
