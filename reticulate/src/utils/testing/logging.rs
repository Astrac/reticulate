use std::sync::Once;

use spdlog::{Level, LevelFilter};

static INIT: Once = Once::new();

/// Routes `log` output through spdlog at trace level, with a compact format.
///
/// Safe to call from any number of tests: tests run on parallel threads of
/// one process, and the global logger can only be installed once.
pub fn init_logging() {
    INIT.call_once(|| {
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
    });
}
