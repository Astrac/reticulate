use crate::{meshing::triangulation::runs::Runs, utils::bitmasks};

const fn walls(columns: &[u32]) -> u32 {
    let mut mask = 0;
    let mut i = 0;
    while i < columns.len() {
        mask |= 1 << columns[i];
        i += 1;
    }
    mask
}

#[test]
fn from_blocks_makes_one_run_per_group_of_blocks() {
    let edge = walls(&[1, 4, 8, 12]);
    let runs = Runs::from_blocks(edge, (1 << 2) | (1 << 9));

    assert_eq!(runs.count(), 2);
    assert_eq!((runs.at_index(0).start(), runs.at_index(0).end()), (1, 4));
    assert_eq!((runs.at_index(1).start(), runs.at_index(1).end()), (8, 12));
    assert_eq!(runs.fill_mask(), 0b1110 | (0b1111 << 8));
    assert_eq!(runs.scan_mask(), 0b11110 | (0b11111 << 8));
}

#[test]
fn blocks_inside_the_same_run_are_merged() {
    let edge = walls(&[1, 4, 8, 12]);
    let runs = Runs::from_blocks(edge, (1 << 1) | (1 << 2) | (1 << 3));

    assert_eq!(runs.count(), 1);
    assert_eq!(runs.fill_mask(), 0b1110);
}

#[test]
fn no_blocks_means_no_runs() {
    let runs = Runs::from_blocks(walls(&[1, 4]), 0);
    assert_eq!(runs.count(), 0);
    assert_eq!(runs.fill_mask(), 0);
    assert_eq!(runs.scan_mask(), 0);
}

#[test]
fn from_columns_mask_seeds_a_single_run() {
    let runs = Runs::from_columns_mask(walls(&[1, 4, 8, 12]), 9);
    assert_eq!(runs.count(), 1);
    assert_eq!((runs.at_index(0).start(), runs.at_index(0).end()), (8, 12));
}

#[test]
fn iterate_runs_visits_runs_in_column_order() {
    let runs = Runs::from_blocks(walls(&[1, 4, 8, 12]), (1 << 9) | (1 << 2));

    let mut starts = Vec::new();
    runs.iterate_runs(|run| starts.push(run.start()));
    assert_eq!(starts, [1, 8]);
}

#[test]
fn all_exposes_the_fixed_size_run_table() {
    let runs = Runs::from_blocks(walls(&[1, 4]), 1 << 2);
    assert_eq!(runs.all().len(), 16);
    assert_eq!(runs.all()[0].start(), 1);
}

#[test]
fn advance_drops_the_columns_the_guard_covers() {
    let edge = walls(&[1, 4, 8, 12]);
    let mut runs = Runs::from_blocks(edge, (1 << 2) | (1 << 9));

    // Guarding the whole first run leaves only the second.
    let guard = runs.at_index(0).fill_mask();
    runs.advance(edge, guard);

    assert_eq!(runs.count(), 1);
    assert_eq!(runs.at_index(0).start(), 8);
}

#[test]
fn advance_adds_guarded_columns_outside_the_current_runs() {
    let edge = walls(&[1, 4, 8, 12]);
    let mut runs = Runs::from_columns_mask(edge, 2);

    // The live columns of the next row are `fill ^ guard`: a guarded column
    // that was not live toggles on.
    runs.advance(edge, 1 << 9);

    assert_eq!(runs.count(), 2);
    assert_eq!(runs.at_index(0).start(), 1);
    assert_eq!(runs.at_index(1).start(), 8);
}

#[test]
fn guard_caps_are_the_segment_boundaries_of_the_guarded_fill() {
    let edge = walls(&[1, 8]);
    let runs = Runs::from_columns_mask(edge, 2); // fill = columns 1..8
    let guard = 0b0011_1000; // columns 3, 4, 5

    assert_eq!(
        runs.guard_caps(guard),
        bitmasks::segment_boundaries(0b0011_1000)
    );
    assert_eq!(runs.guard_caps(guard), (1 << 3) | (1 << 6));
    assert_eq!(runs.guard_caps(0), 0);
}
