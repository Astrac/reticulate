use crate::{Coords, Octree, octree::Node};

use spdlog::{Level, LevelFilter};

// TODO: Check if simpler using env_logger or anythign else
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
fn log_octree<Data: std::fmt::Debug + Clone + PartialEq>(octree: &Octree<Data>, message: &str) {
    let mut tokens = Vec::with_capacity(octree.nodes().len());

    for (idx, n) in octree.nodes().iter().enumerate() {
        match n {
            Node::Branch { children_index } => {
                tokens.push(format!("\t{idx:0>8}: Branch: {children_index}"));
            }
            Node::Leaf { data } => {
                tokens.push(format!("\t{idx:0>8}: Leaf: {data:?}"));
            }
        }
    }

    log::trace!("{}: \n{}", message, tokens.join("\n"));
}

#[test]
fn test_new_creates_empty_octree() {
    init_logging();
    let octree: Octree<i32> = Octree::new();
    let coords = Coords { x: 0, y: 0, z: 0 };
    assert!(octree.get(coords).is_none());
}

#[test]
fn test_update_and_get_basic() {
    init_logging();
    let mut octree = Octree::new();
    let coords = Coords {
        x: 10,
        y: 20,
        z: 30,
    };
    let value = 42;

    // Updated: Wrap data in Some
    octree.update(coords, Some(value));

    assert_eq!(octree.get(coords), Some(&42));
}

#[test]
fn test_update_overwrites_existing_data() {
    init_logging();
    let mut octree = Octree::new();
    let coords = Coords { x: 5, y: 5, z: 5 };

    // Updated: Wrap data in Some
    octree.update(coords, Some(1));
    assert_eq!(octree.get(coords), Some(&1));

    // Updated: Wrap data in Some
    octree.update(coords, Some(2));
    assert_eq!(octree.get(coords), Some(&2));
}

#[test]
fn test_remove_existing_data() {
    init_logging();
    let mut octree = Octree::new();
    let coords = Coords {
        x: 10,
        y: 20,
        z: 30,
    };

    // Insert data
    octree.update(coords, Some(42));
    assert_eq!(octree.get(coords), Some(&42));

    // Remove data using None
    octree.update(coords, None);
    assert!(octree.get(coords).is_none());
}

#[test]
fn test_remove_non_existent_data() {
    init_logging();
    let mut octree = Octree::<u32>::new();
    let coords = Coords {
        x: 10,
        y: 20,
        z: 30,
    };

    // Attempting to remove data that doesn't exist should be safe
    octree.update(coords, None);
    assert!(octree.get(coords).is_none());
}

#[test]
fn test_remove_and_reinsert() {
    init_logging();
    let mut octree = Octree::new();
    let coords = Coords { x: 5, y: 5, z: 5 };

    // Insert initial data
    octree.update(coords, Some(1));
    assert_eq!(octree.get(coords), Some(&1));

    // Remove
    octree.update(coords, None);
    assert!(octree.get(coords).is_none());

    // Re-insert with new data
    octree.update(coords, Some(2));
    assert_eq!(octree.get(coords), Some(&2));
}

#[test]
fn test_distinct_coordinates_are_independent() {
    init_logging();
    let mut octree = Octree::new();

    let c1 = Coords { x: 0, y: 0, z: 0 };
    let c2 = Coords { x: 1, y: 1, z: 1 };

    // Updated: Wrap data in Some
    octree.update(c1, Some("Data A"));
    log_octree(&octree, "A");
    // Updated: Wrap data in Some
    octree.update(c2, Some("Data B"));
    log_octree(&octree, "B");

    assert_eq!(octree.get(c1), Some(&"Data A"));
    assert_eq!(octree.get(c2), Some(&"Data B"));
}

#[test]
fn test_boundary_coordinates() {
    init_logging();
    let mut octree = Octree::new();

    let min_coords = Coords { x: 0, y: 0, z: 0 };
    // Updated: Wrap data in Some
    octree.update(min_coords, Some("Min"));
    log_octree(&octree, "After Min:");
    assert_eq!(octree.get(min_coords), Some(&"Min"));

    let max_coords = Coords {
        x: 255,
        y: 255,
        z: 255,
    };
    // Updated: Wrap data in Some
    octree.update(max_coords, Some("Max"));
    log_octree(&octree, "After Max:");
    assert_eq!(octree.get(max_coords), Some(&"Max"));
}

#[test]
fn test_generic_data_types() {
    init_logging();

    #[derive(Clone, PartialEq, Debug)]
    struct CustomData {
        val: u32,
    }

    let mut octree: Octree<CustomData> = Octree::new();
    let coords = Coords {
        x: 100,
        y: 100,
        z: 100,
    };
    let data = CustomData { val: 999 };

    // Updated: Wrap data in Some
    octree.update(coords, Some(data));

    let retrieved = octree.get(coords);
    assert_eq!(retrieved.map(|r| r.val), Some(999));
}
