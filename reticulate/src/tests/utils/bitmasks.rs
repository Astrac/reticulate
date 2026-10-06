use crate::utils::bitmasks::*;

#[test]
fn take_one_every_3_bits_picks_the_lowest_bit_of_each_triplet() {
    assert_eq!(take_one_every_3_bits(0b100_100_100), 0b000);
    assert_eq!(take_one_every_3_bits(0b010_010_010), 0b000);
    assert_eq!(take_one_every_3_bits(0b001_001_001), 0b111);
    assert_eq!(take_one_every_3_bits(0b100_010_001), 0b001);
    assert_eq!(take_one_every_3_bits(0), 0);
}

#[test]
fn interleave_zeroes_spreads_bits_to_even_positions() {
    assert_eq!(interleave_zeroes(0b111), 0b010101);
    assert_eq!(interleave_zeroes(0b101), 0b010001);
    assert_eq!(interleave_zeroes(0b010), 0b000100);
    assert_eq!(interleave_zeroes(0b001), 0b000001);
    assert_eq!(interleave_zeroes(0), 0);
}

#[test]
fn interleave_zeroes_uses_the_low_16_bits_only() {
    assert_eq!(interleave_zeroes(0xFFFF), 0x5555_5555);
    assert_eq!(interleave_zeroes(0xFFFF_0000), 0);
    assert_eq!(interleave_zeroes(0x1_0001), 1);
}

#[test]
fn take_one_every_2_bits_picks_the_even_bits() {
    assert_eq!(take_one_every_2_bits(0b101010), 0b000);
    assert_eq!(take_one_every_2_bits(0b010101), 0b111);
    assert_eq!(take_one_every_2_bits(0b100100), 0b010);
    assert_eq!(take_one_every_2_bits(0), 0);
}

#[test]
fn take_one_every_2_bits_undoes_interleave_zeroes() {
    for x in (0..=0xFFFF).step_by(7).chain([0xFFFF]) {
        assert_eq!(take_one_every_2_bits(interleave_zeroes(x)), x);
    }
}

#[test]
fn interleave_two_zeroes_spreads_bits_to_every_third_position() {
    assert_eq!(interleave_two_zeroes(0b111_u8), 0b001_001_001);
    assert_eq!(interleave_two_zeroes(0b101_u8), 0b001_000_001);
    assert_eq!(interleave_two_zeroes(0b010_u8), 0b000_001_000);
    assert_eq!(interleave_two_zeroes(0b001_u8), 0b000_000_001);
    assert_eq!(interleave_two_zeroes(0), 0);
    assert_eq!(interleave_two_zeroes(u8::MAX), 0x0024_9249);
}

#[test]
fn take_one_every_3_bits_undoes_interleave_two_zeroes() {
    for x in 0..=u8::MAX {
        assert_eq!(take_one_every_3_bits(interleave_two_zeroes(x)), x as u32);
    }
}

#[test]
fn make_ones_mask_is_inclusive_on_both_ends() {
    assert_eq!(cell_ones(3, 5), 0b111_000);
    assert_eq!(cell_ones(0, 0), 0b1);
    assert_eq!(cell_ones(4, 4), 0b1_0000);
    assert_eq!(cell_ones(0, 3), 0b1111);
}

#[test]
fn make_ones_mask_reaches_the_top_bit() {
    assert_eq!(cell_ones(0, 31), u32::MAX);
    assert_eq!(cell_ones(31, 31), 1 << 31);
    assert_eq!(cell_ones(28, 31), 0xF000_0000);
}

#[test]
fn run_starts_and_ends_mark_each_run_once() {
    // Cells 3 and 4: starts at 3, ends at 5.
    assert_eq!(run_starts(0b11000), 1 << 3);
    assert_eq!(run_ends(0b11000), 1 << 5);

    // Two runs, cells {1} and {3, 4}.
    let cells = 0b11010;
    assert_eq!(run_starts(cells), (1 << 1) | (1 << 3));
    assert_eq!(run_ends(cells), (1 << 2) | (1 << 5));
}

#[test]
fn a_run_touching_cell_31_ends_at_boundary_32() {
    assert_eq!(run_ends(1 << 31), 1u64 << 32);
    assert_eq!(run_starts(1 << 31), 1u64 << 31);
    assert_eq!(segment_boundaries(1 << 31), (1u64 << 31) | (1u64 << 32));
}

#[test]
fn a_full_row_spans_boundaries_0_to_32() {
    assert_eq!(run_starts(u32::MAX), 1);
    assert_eq!(run_ends(u32::MAX), 1u64 << 32);
    assert_eq!(segment_boundaries(u32::MAX), 1 | (1u64 << 32));
}

#[test]
fn moving_the_end_onto_boundary_32_is_a_change() {
    // cells {30} -> {30, 31}: the start stays at 30, the end moves from 31 to 32.
    assert_eq!(
        boundary_changes(1 << 30, 0b11 << 30),
        (1u64 << 31) | (1u64 << 32)
    );
}

#[test]
fn empty_and_full_rows_have_no_inner_boundaries() {
    assert_eq!(run_starts(0), 0);
    assert_eq!(run_ends(0), 0);
    assert_eq!(segment_boundaries(0), 0);
    assert_eq!(run_starts(u32::MAX), 1);
}

#[test]
fn segment_boundaries_are_run_starts_and_ends() {
    assert_eq!(segment_boundaries(0b11000), (1 << 3) | (1 << 5));

    let cells = 0b11010;
    assert_eq!(
        segment_boundaries(cells),
        run_starts(cells) | run_ends(cells)
    );
}

#[test]
fn boundary_changes_ignore_straight_walls() {
    // A straight wall at boundary 1 on both rows: no change.
    assert_eq!(boundary_changes(0b0110, 0b0110), 0);
    assert_eq!(boundary_changes(0, 0), 0);
}

#[test]
fn boundary_changes_keep_pinches_that_xor_would_cancel() {
    // Pinch: boundary 3 starts a run below ({3}) and ends one above ({1, 2}).
    // A plain XOR of the edges cancels it; boundary_changes keeps it.
    let (below, above) = (0b1000, 0b0110);
    assert_eq!(
        segment_boundaries(below) ^ segment_boundaries(above),
        (1 << 1) | (1 << 4)
    );
    assert_eq!(
        boundary_changes(below, above),
        (1 << 1) | (1 << 3) | (1 << 4)
    );
}

#[test]
fn boundary_changes_report_new_and_vanishing_runs() {
    // A run only above, cells {2, 3}: both of its boundaries change.
    assert_eq!(boundary_changes(0, 0b1100), (1 << 2) | (1 << 4));
    // The same run only below is symmetric.
    assert_eq!(boundary_changes(0b1100, 0), (1 << 2) | (1 << 4));
}

#[test]
fn boundary_points_of_a_run_against_nothing_cover_its_whole_extent() {
    // Cells {1, 2} above, nothing below: the row change puts vertices on
    // columns 1..=3, and the run's own boundaries are a subset.
    assert_eq!(boundary_points(0, 0b110), 0b1110);
    assert_eq!(boundary_points(0b110, 0), 0b1110);
}

#[test]
fn boundary_points_are_empty_for_empty_rows() {
    assert_eq!(boundary_points(0, 0), 0);
}

#[test]
fn boundary_points_include_the_boundaries_of_both_rows() {
    let (below, above) = (0b0011_1100, 0b0011_1100);
    assert_eq!(
        boundary_points(below, above),
        (1 << 2) | (1 << 6),
        "identical rows only expose their shared boundaries"
    );
}
