use crate::{Coords, Octree};

#[test]
fn test_new_creates_empty_octree() {
    super::init_logging();
    let octree: Octree<i32> = Octree::new();
    let coords = Coords { x: 0, y: 0, z: 0 };
    assert!(octree.get(coords).is_none());
}

#[test]
fn test_update_and_get_basic() {
    super::init_logging();
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
    super::init_logging();
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
    super::init_logging();
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
    super::init_logging();
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
    super::init_logging();
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
    super::init_logging();
    let mut octree = Octree::new();

    let c1 = Coords { x: 0, y: 0, z: 0 };
    let c2 = Coords { x: 1, y: 1, z: 1 };

    // Updated: Wrap data in Some
    octree.update(c1, Some("Data A"));
    // Updated: Wrap data in Some
    octree.update(c2, Some("Data B"));

    assert_eq!(octree.get(c1), Some(&"Data A"));
    assert_eq!(octree.get(c2), Some(&"Data B"));
}

#[test]
fn test_boundary_coordinates() {
    super::init_logging();
    let mut octree = Octree::new();

    let min_coords = Coords { x: 0, y: 0, z: 0 };
    // Updated: Wrap data in Some
    octree.update(min_coords, Some("Min"));
    assert_eq!(octree.get(min_coords), Some(&"Min"));

    let max_coords = Coords {
        x: 255,
        y: 255,
        z: 255,
    };
    // Updated: Wrap data in Some
    octree.update(max_coords, Some("Max"));
    assert_eq!(octree.get(max_coords), Some(&"Max"));
}

#[test]
fn test_generic_data_types() {
    super::init_logging();

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
