use crate::meshing::triangulation::runs::Runs;

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
fn iterate_runs_visits_runs_in_column_order() {
    let runs = Runs::from_blocks(walls(&[1, 4, 8, 12]), (1 << 9) | (1 << 2));

    let mut starts = Vec::new();
    runs.iterate_runs(|run| starts.push(run.start()));
    assert_eq!(starts, [1, 8]);
}
