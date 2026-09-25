mod compact;
mod fixed8;
mod material_plane;
mod material_plane2;
mod morton;
mod octree;
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
