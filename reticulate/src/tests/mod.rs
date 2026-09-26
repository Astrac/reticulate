mod claims;
mod compact;
mod fixed8;
mod material_plane;
mod morton;
mod octree;
mod parity_bitfield;
mod strip_weaver;

use spdlog::{Level, LevelFilter};

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
