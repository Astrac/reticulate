mod bitfields;
mod morton;
mod octree;
mod scanner;
mod shaper;
mod stepper;

use spdlog::{Level, LevelFilter};

use crate::meshing::triangulation::Triangle2D;

// TODO: Check if simpler using env_logger or anythign else; avoid multiple inits
pub fn init_logging() {
    spdlog::default_logger().set_level_filter(LevelFilter::MoreSevereEqual(Level::Trace));
    spdlog::init_log_crate_proxy().expect("Failed to initialize spdlog proxy");
    spdlog::re_export::log::set_max_level(spdlog::re_export::log::LevelFilter::Trace);

    let formatter = spdlog::formatter::FullFormatter::builder()
        .time(false)
        .source_location(false)
        .logger_name(false)
        .build();

    for sink in spdlog::default_logger().sinks() {
        sink.set_formatter(Box::new(formatter.clone()));
    }
}

#[allow(dead_code)]
#[cfg(any(test, debug_assertions))]
pub fn find_unclaimed_faces(
    parity: &crate::meshing::bitfields::Parity,
    occlusion: &crate::meshing::bitfields::Claims,
) -> Vec<(crate::axis::Axis, usize, crate::Coords2D)> {
    let mut missing = Vec::new();
    for axis in [
        crate::axis::AXIS_X,
        crate::axis::AXIS_Y,
        crate::axis::AXIS_Z,
    ] {
        for axis_coord in 0..crate::meshing::bitfields::PLANE_SIDE {
            for u in 0..crate::meshing::bitfields::PLANE_SIDE {
                let mut bits = parity.gather_footprint(axis, axis_coord, u);
                while bits != 0 {
                    let v = bits.trailing_zeros();
                    bits &= bits - 1;
                    let coords = crate::Coords2D { u: u as u32, v };
                    if occlusion.is_pending(axis, axis_coord, coords) {
                        missing.push((axis, axis_coord, coords));
                    }
                }
            }
        }
    }
    missing
}

pub fn assert_covers_exactly(tris: &[Triangle2D], expected: &[u32]) {
    let counts = coverage(tris);
    for row in 0..32 {
        for col in 0..32 {
            let want = expected.get(row).map_or(0, |m| (m >> col) & 1) as u8;
            assert_eq!(
                counts[row][col], want,
                "cell (col {col}, row {row}) covered {} times, expected {want}",
                counts[row][col]
            );
        }
    }
}

fn coverage(tris: &[Triangle2D]) -> [[u8; 32]; 32] {
    let mut counts = [[0u8; 32]; 32]; // [row][col]
    for t in tris {
        let p: Vec<(f64, f64)> = t
            .vertices
            .iter()
            .map(|&(c, r)| (c as f64, r as f64))
            .collect();
        let area2 = (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[2].0 - p[0].0) * (p[1].1 - p[0].1);
        assert!(area2 != 0.0, "degenerate triangle: {:?}", t);
        for row in 0..32 {
            for col in 0..32 {
                let (x, y) = (col as f64 + 0.5 + 1e-4, row as f64 + 0.5 + 1.414e-4);
                let side = |a: (f64, f64), b: (f64, f64)| {
                    (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0)
                };
                let (s0, s1, s2) = (side(p[0], p[1]), side(p[1], p[2]), side(p[2], p[0]));
                if (s0 > 0.0 && s1 > 0.0 && s2 > 0.0) || (s0 < 0.0 && s1 < 0.0 && s2 < 0.0) {
                    counts[row][col] += 1;
                }
            }
        }
    }
    counts
}
