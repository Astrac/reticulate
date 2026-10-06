use crate::{
    BoundaryMask,
    meshing::triangulation::{Triangle2D, stepper::BOUNDARIES_COUNT},
};

const INACTIVE_VERTEX_U32: u32 = u32::MAX;

// TODO - Make the attributes private if possible
#[derive(Clone, Debug)]
pub struct Zipper {
    pub bottom_column_heights: [u8; BOUNDARIES_COUNT],
    pub bottom_columns_mask: BoundaryMask,
}

impl Default for Zipper {
    fn default() -> Self {
        Self {
            bottom_column_heights: [Default::default(); BOUNDARIES_COUNT],
            bottom_columns_mask: Default::default(),
        }
    }
}

impl Zipper {
    pub fn zip(
        &mut self,
        run_scan_mask: BoundaryMask,
        top_height: usize,
        top_columns_mask: BoundaryMask,
        on_triangle: &mut impl FnMut(Triangle2D),
    ) {
        let mut top_mask = top_columns_mask & run_scan_mask;
        let mut bottom_mask = self.bottom_columns_mask & run_scan_mask;

        let (Some(mut top), Some(mut bottom)) = (top_mask.lowest_one(), bottom_mask.lowest_one())
        else {
            return;
        };

        top_mask &= top_mask - 1;
        bottom_mask &= bottom_mask - 1;

        let height = |column: u32| self.bottom_column_heights[column as usize];

        loop {
            let (next_top, next_bottom) = (top_mask.lowest_one(), bottom_mask.lowest_one());
            let next_top = next_top.unwrap_or(INACTIVE_VERTEX_U32);
            let next_bottom = next_bottom.unwrap_or(INACTIVE_VERTEX_U32);

            if next_top == INACTIVE_VERTEX_U32 && next_bottom == INACTIVE_VERTEX_U32 {
                break;
            }

            let diagonal_ahead =
                next_bottom != INACTIVE_VERTEX_U32 && height(next_bottom) < height(bottom);

            if next_top != INACTIVE_VERTEX_U32 && (next_top < next_bottom || diagonal_ahead) {
                top_mask &= top_mask - 1;
                on_triangle(Triangle2D {
                    vertices: [
                        (top as u8, top_height as u8),
                        (bottom as u8, height(bottom)),
                        (next_top as u8, top_height as u8),
                    ],
                });
                top = next_top;
            } else if diagonal_ahead && bottom > top {
                break;
            } else {
                let skip = !diagonal_ahead && next_bottom < top;
                bottom_mask &= bottom_mask - 1;
                if !skip {
                    self.bottom_columns_mask &= !(1 << next_bottom);
                    on_triangle(Triangle2D {
                        vertices: [
                            (top as u8, top_height as u8),
                            (bottom as u8, height(bottom)),
                            (next_bottom as u8, height(next_bottom)),
                        ],
                    });
                }
                bottom = next_bottom;
            }
        }

        self.bottom_columns_mask |= 1 << bottom;
    }

    #[inline(always)]
    pub const fn advance(&mut self, row: usize, mut mask: BoundaryMask, scan_mask: BoundaryMask) {
        self.bottom_columns_mask = (self.bottom_columns_mask & !mask) | (mask & scan_mask);
        while let Some(x) = mask.lowest_one() {
            mask &= !mask.isolate_lowest_one();
            self.bottom_column_heights[x as usize] = row as u8;
        }
    }
}
