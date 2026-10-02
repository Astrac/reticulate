use crate::meshing::triangulation::run_mask::RunMasks;

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
