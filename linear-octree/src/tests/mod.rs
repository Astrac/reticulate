mod block_faces;
mod morton;
mod octree;

use spdlog::{Level, LevelFilter};

use crate::{Octree, octree::OctreeNode};

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
pub fn log_octree<Data: std::fmt::Debug + Clone + PartialEq>(octree: &Octree<Data>, message: &str) {
    let mut tokens = Vec::with_capacity(octree.nodes().len());

    for (idx, n) in octree.nodes().iter().enumerate() {
        match n {
            OctreeNode::Branch { children_index } => {
                tokens.push(format!("\t{idx:0>8}: Branch: {children_index}"));
            }
            OctreeNode::Leaf { data } => {
                tokens.push(format!("\t{idx:0>8}: Leaf: {data:?}"));
            }
        }
    }

    log::trace!("{}: \n{}", message, tokens.join("\n"));
}
