use crate::meshing::triangulation::run::Run;

/// Walls at columns 0, 4 and 7.
const WALLS: u32 = 0b1001_0001;

#[test]
fn run_is_bounded_by_the_walls_around_the_seed() {
    let run = Run::from_columns_mask(WALLS, 2);
    assert_eq!((run.start(), run.end()), (0, 4));
}

#[test]
fn run_covers_the_columns_between_its_walls() {
    let run = Run::from_columns_mask(WALLS, 2);
    assert_eq!(run.fill_mask(), 0b0_1111, "columns 0..4");
    assert_eq!(run.scan_mask(), 0b1_1111, "columns 0..=4, walls included");
}

#[test]
fn boundary_mask_marks_both_walls() {
    let run = Run::from_columns_mask(WALLS, 2);
    assert_eq!(run.boundary_mask(), 0b1_0001);
}

#[test]
fn a_seed_on_a_wall_starts_the_run_there() {
    let run = Run::from_columns_mask(WALLS, 4);
    assert_eq!((run.start(), run.end()), (4, 7));
    assert_eq!(run.fill_mask(), 0b111_0000);
    assert_eq!(run.scan_mask(), 0b1111_0000);
}

#[test]
fn the_run_right_of_the_last_wall_pair_is_bounded_by_them() {
    let run = Run::from_columns_mask(WALLS, 5);
    assert_eq!((run.start(), run.end()), (4, 7));
}

#[test]
fn seeds_in_the_same_run_give_the_same_run() {
    let a = Run::from_columns_mask(WALLS, 1);
    let b = Run::from_columns_mask(WALLS, 3);
    assert_eq!((a.start(), a.end()), (b.start(), b.end()));
    assert_eq!(a.fill_mask(), b.fill_mask());
}

#[test]
fn adjacent_walls_make_a_one_column_run() {
    let run = Run::from_columns_mask(0b0110, 1);
    assert_eq!((run.start(), run.end()), (1, 2));
    assert_eq!(run.fill_mask(), 0b0010);
    assert_eq!(run.scan_mask(), 0b0110);
}

#[test]
fn default_run_is_empty() {
    let run = Run::default();
    assert_eq!((run.start(), run.end()), (0, 0));
    assert_eq!(run.fill_mask(), 0);
}
