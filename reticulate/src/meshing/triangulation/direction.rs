use crate::meshing::{bitfields::PLANE_SIDE, triangulation::Triangle2D};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    #[inline(always)]
    pub fn plane_row(self, r: usize) -> Option<usize> {
        if r >= PLANE_SIDE {
            return None;
        }
        Some(match self {
            Direction::Up => r,
            Direction::Down => PLANE_SIDE - 1 - r,
        })
    }

    #[inline(always)]
    pub fn boundary(self, r: usize) -> usize {
        match self {
            Direction::Up => r,
            Direction::Down => PLANE_SIDE - r,
        }
    }

    #[inline(always)]
    pub fn map(self, t: Triangle2D) -> Triangle2D {
        match self {
            Direction::Up => t,
            Direction::Down => {
                let m = |(col, line): (u8, u8)| (col, PLANE_SIDE as u8 - line);
                let [a, b, c] = t.vertices;
                Triangle2D {
                    vertices: [m(a), m(c), m(b)],
                    ..t
                }
            }
        }
    }
}
