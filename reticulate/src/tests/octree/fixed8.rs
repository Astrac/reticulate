use crate::{Coords3D, Fixed8Octree, utils::testing::init_logging};

#[test]
fn new_initializes_with_root() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    assert!(
        !tree.nodes().is_empty(),
        "New tree should have at least a root node"
    );
}

#[test]
fn update_inserts_and_retrieves_data() {
    let mut tree = Fixed8Octree::new();
    let coords = Coords3D {
        x: 10,
        y: 20,
        z: 30,
    };
    let data = "test_value".to_string();

    tree.update(coords, Some(data.clone()));

    assert_eq!(tree.get(coords), Some(&data));
}

#[test]
fn update_removes_data() {
    let mut tree = Fixed8Octree::new();
    let coords = Coords3D { x: 0, y: 0, z: 0 };

    tree.update(coords, Some("data".to_string()));
    assert!(tree.get(coords).is_some());

    tree.update(coords, None);
    assert_eq!(tree.get(coords), None);
}

#[test]
fn get_returns_none_for_missing() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    let coords = Coords3D {
        x: 100,
        y: 100,
        z: 100,
    };

    assert_eq!(tree.get(coords), None);
}

#[test]
fn new_creates_empty_octree() {
    init_logging();
    let octree: Fixed8Octree<i32> = Fixed8Octree::new();
    let coords = Coords3D { x: 0, y: 0, z: 0 };
    assert!(octree.get(coords).is_none());
}

#[test]
fn update_and_get_basic() {
    init_logging();
    let mut octree = Fixed8Octree::new();
    let coords = Coords3D {
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
fn update_overwrites_existing_data() {
    init_logging();
    let mut octree = Fixed8Octree::new();
    let coords = Coords3D { x: 5, y: 5, z: 5 };

    // Updated: Wrap data in Some
    octree.update(coords, Some(1));
    assert_eq!(octree.get(coords), Some(&1));

    // Updated: Wrap data in Some
    octree.update(coords, Some(2));
    assert_eq!(octree.get(coords), Some(&2));
}

#[test]
fn remove_existing_data() {
    init_logging();
    let mut octree = Fixed8Octree::new();
    let coords = Coords3D {
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
fn remove_non_existent_data() {
    init_logging();
    let mut octree = Fixed8Octree::<u32>::new();
    let coords = Coords3D {
        x: 10,
        y: 20,
        z: 30,
    };

    // Attempting to remove data that doesn't exist should be safe
    octree.update(coords, None);
    assert!(octree.get(coords).is_none());
}

#[test]
fn remove_and_reinsert() {
    init_logging();
    let mut octree = Fixed8Octree::new();
    let coords = Coords3D { x: 5, y: 5, z: 5 };

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
fn distinct_coordinates_are_independent() {
    init_logging();
    let mut octree = Fixed8Octree::new();

    let c1 = Coords3D { x: 0, y: 0, z: 0 };
    let c2 = Coords3D { x: 1, y: 1, z: 1 };

    // Updated: Wrap data in Some
    octree.update(c1, Some("Data A"));
    // Updated: Wrap data in Some
    octree.update(c2, Some("Data B"));

    assert_eq!(octree.get(c1), Some(&"Data A"));
    assert_eq!(octree.get(c2), Some(&"Data B"));
}

#[test]
fn boundary_coordinates() {
    init_logging();
    let mut octree = Fixed8Octree::new();

    let min_coords = Coords3D { x: 0, y: 0, z: 0 };
    // Updated: Wrap data in Some
    octree.update(min_coords, Some("Min"));
    assert_eq!(octree.get(min_coords), Some(&"Min"));

    let max_coords = Coords3D {
        x: 255,
        y: 255,
        z: 255,
    };
    // Updated: Wrap data in Some
    octree.update(max_coords, Some("Max"));
    assert_eq!(octree.get(max_coords), Some(&"Max"));
}

#[test]
fn generic_data_types() {
    init_logging();

    #[derive(Clone, PartialEq, Debug)]
    struct CustomData {
        val: u32,
    }

    let mut octree: Fixed8Octree<CustomData> = Fixed8Octree::new();
    let coords = Coords3D {
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
