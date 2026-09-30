use crate::meshing::triangulation::{run::Run, run_mask::RunMasks};

#[test]
fn new_keeps_fill_and_scan() {
    let masks = RunMasks::new(0b0110, 0b1111);
    assert_eq!(masks.fill(), 0b0110);
    assert_eq!(masks.scan(), 0b1111);
}

#[test]
fn default_is_empty() {
    let masks = RunMasks::default();
    assert_eq!((masks.fill(), masks.scan()), (0, 0));
}

#[test]
fn from_runs_unions_the_first_count_runs() {
    let mut runs = [Run::default(); 16];
    runs[0] = Run::from_columns_mask(0b0_0001_0001, 2); // columns 0..4
    runs[1] = Run::from_columns_mask(0b1_0001_0000, 5); // columns 4..8
    runs[2] = Run::from_columns_mask(0b1_0000_0000 | 1 << 12, 10); // columns 8..12

    let first_two = RunMasks::from_runs(&runs, 2);
    assert_eq!(first_two.fill(), runs[0].fill_mask() | runs[1].fill_mask());
    assert_eq!(first_two.scan(), runs[0].scan_mask() | runs[1].scan_mask());

    let none = RunMasks::from_runs(&runs, 0);
    assert_eq!((none.fill(), none.scan()), (0, 0));

    let all = RunMasks::from_runs(&runs, 3);
    assert_eq!(all.fill(), 0b1111_1111_1111);
}
