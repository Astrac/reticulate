mod compact;
mod faces;
mod fixed8;
mod morton;
mod octree;

use spdlog::{Level, LevelFilter};

// use crate::{Fixed8Octree, octree::Fixed8OctreeNode};

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

// #[allow(dead_code)]
// pub fn log_octree<Data: std::fmt::Debug + Clone + PartialEq>(octree: &Fixed8Octree<Data>, message: &str) {
//     let mut tokens = Vec::with_capacity(octree.nodes().len());
//
//     for (idx, n) in octree.nodes().iter().enumerate() {
//         match n {
//             Fixed8OctreeNode::Branch { children_index, children_mask: .. } => {
//                 tokens.push(format!("\t{idx:0>8}: Branch: {children_index}"));
//             }
//             Fixed8OctreeNode::Leaf { data } => {
//                 tokens.push(format!("\t{idx:0>8}: Leaf: {data:?}"));
//             }
//         }
//     }
//
//     log::trace!("{}: \n{}", message, tokens.join("\n"));
// }
