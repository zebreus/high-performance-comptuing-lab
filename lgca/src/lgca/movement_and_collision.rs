use super::cell::Cell;

use crate::lgca::cell::{
    TO_EAST, TO_NORTH_EAST, TO_NORTH_WEST, TO_SOUTH_EAST, TO_SOUTH_WEST, TO_WEST,
};

pub struct Section<const WIDTH: usize, const HEIGHT: usize>
where
    [(); (WIDTH) * (HEIGHT)]:,
{
    pub data: Box<[Cell; (WIDTH) * (HEIGHT)]>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Section<WIDTH, HEIGHT>
where
    [(); (WIDTH) * (HEIGHT)]:,
{
    pub fn new() -> Self {
        Self {
            data: unsafe { Box::new_zeroed().assume_init() },
        }
    }
}

pub struct SectionContext<const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH + 1]:,
    [(); HEIGHT]:,
{
    pub above: Box<[Cell; WIDTH + 1]>,
    pub before: Box<[Cell; HEIGHT]>,
    pub below: Box<[Cell; WIDTH + 1]>,
    pub after: Box<[Cell; HEIGHT]>,
}

pub struct InputContext<const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH]:,
    [(); HEIGHT]:,
{
    pub above: Box<[Cell; WIDTH]>,
    pub before: Box<[Cell; HEIGHT]>,
    pub below: Box<[Cell; WIDTH]>,
    pub after: Box<[Cell; HEIGHT]>,
}

// Get out propagation
// Share out prop
// Inside propagation + collision

pub fn get_out_propagation<const WIDTH: usize, const HEIGHT: usize>(
    section: &[Cell; WIDTH * HEIGHT],
) -> SectionContext<WIDTH, HEIGHT>
where
    [(); WIDTH + 1]:,
    [(); HEIGHT]:,
{
    let mut out_above: Box<[Cell; WIDTH + 1]> = unsafe { Box::new_zeroed().assume_init() };
    let mut out_before: Box<[Cell; HEIGHT]> = unsafe { Box::new_zeroed().assume_init() };
    let mut out_below: Box<[Cell; WIDTH + 1]> = unsafe { Box::new_zeroed().assume_init() };
    let mut out_after: Box<[Cell; HEIGHT]> = unsafe { Box::new_zeroed().assume_init() };
    // let mut row_refs: [&[Cell; WIDTH + 2]; HEIGHT + 2] = [&[]; HEIGHT + 2];

    for (x, cell) in section[0..WIDTH].iter().enumerate() {
        out_above[x].receive_from_south_east(cell);
        out_above[x + 1].receive_from_south_west(cell);
    }
    for (x, cell) in section[((HEIGHT - 1) * WIDTH)..(HEIGHT * WIDTH)]
        .iter()
        .enumerate()
    {
        out_below[x].receive_from_north_east(cell);
        out_below[x + 1].receive_from_north_west(cell);
    }

    for (x, cell) in section
        .array_chunks::<WIDTH>()
        .map(|row| row.first().unwrap())
        .enumerate()
    {
        out_before[x].receive_from_east(cell);
        if x % 2 == 1 {
            if x != 0 {
                out_before[x - 1].receive_from_south_east(cell);
            }
            if x != HEIGHT - 1 {
                out_before[x + 1].receive_from_north_east(cell);
            }
        }
    }
    for (x, cell) in section
        .array_chunks::<WIDTH>()
        .map(|row| row.last().unwrap())
        .enumerate()
    {
        out_after[x].receive_from_west(cell);
        if x % 2 == 0 {
            if x != HEIGHT - 1 {
                out_after[x + 1].receive_from_north_west(cell);
            }
            if x != 0 {
                out_after[x - 1].receive_from_south_west(cell);
            }
        }
    }

    SectionContext {
        above: out_above,
        before: out_before,
        below: out_below,
        after: out_after,
    }
}

pub fn get_in_propagation<const WIDTH: usize, const HEIGHT: usize>(
    section: &mut [Cell; WIDTH * HEIGHT],
    context: &SectionContext<WIDTH, HEIGHT>,
) where
    [(); WIDTH * HEIGHT]:,
    [(); WIDTH + 1]:,
    [(); HEIGHT]:,
{
    let SectionContext {
        before,
        after,
        above,
        below,
    } = context;

    let mut overflow_from_above_row: [Cell; WIDTH + 1] = above.as_ref().clone();
    // let overflow_from_before: &Cell =

    let rows_iterator = section.array_chunks_mut::<WIDTH>().zip(before.iter());
    for (row_index, (row, before)) in rows_iterator.enumerate() {
        let mut overflow_from_above_iter = overflow_from_above_row.iter_mut();
        let mut current_overflow_from_above = overflow_from_above_iter.next().unwrap();
        let mut overflow_from_before = before.clone();
        if row_index % 2 == 0 {
            current_overflow_from_above.raw = 0;
            for cell in row.iter_mut() {
                let next_overflow_from_above = overflow_from_above_iter.next().unwrap();
                let new_raw = next_overflow_from_above.raw | overflow_from_before.raw;
                overflow_from_before.raw = cell.raw & TO_EAST;
                next_overflow_from_above.raw = cell.raw & (TO_SOUTH_EAST);
                current_overflow_from_above.raw |= cell.raw & (TO_SOUTH_WEST);
                cell.raw = new_raw | (cell.raw & (TO_NORTH_EAST | TO_NORTH_WEST | TO_WEST));

                current_overflow_from_above = next_overflow_from_above;
            }
        } else {
            let mut temp: u8 = current_overflow_from_above.raw;
            for cell in row.iter_mut() {
                let next_overflow_from_above = overflow_from_above_iter.next().unwrap();

                let new_raw = temp | overflow_from_before.raw;
                temp = next_overflow_from_above.raw;
                overflow_from_before.raw = cell.raw & TO_EAST;
                next_overflow_from_above.raw = cell.raw & (TO_SOUTH_EAST);
                current_overflow_from_above.raw |= cell.raw & (TO_SOUTH_WEST);
                cell.raw = new_raw | (cell.raw & (TO_NORTH_EAST | TO_NORTH_WEST | TO_WEST));

                current_overflow_from_above = next_overflow_from_above;
            }
        }

        // for (cell_index, cell) in row.iter().enumerate() {
        //     cell.receive_from_north_west(&overflow_from_above[cell_index]);
        //     cell.receive_from_north_east(&overflow_from_above[cell_index + 1]);
        //     cell.receive_from_west(&before[row_index]);
        //     cell.receive_from_east(&after[row_index]);
        //     cell.receive_from_south_west(&below[cell_index]);
        //     cell.receive_from_south_east(&below[cell_index + 1]);
        // }
    }

    let mut overflow_from_below_row: [Cell; WIDTH + 1] = below.as_ref().clone();
    // let overflow_from_before: &Cell =

    let rows_iterator = section.array_chunks_mut::<WIDTH>().zip(after.iter());
    for (row_index, (row, before)) in rows_iterator.enumerate().rev() {
        let mut overflow_from_below_iter = overflow_from_below_row.iter_mut().rev();
        let mut current_overflow_from_above = overflow_from_below_iter.next().unwrap();
        let mut overflow_from_before = before.clone();
        if row_index % 2 == 1 {
            current_overflow_from_above.raw = 0;
            for cell in row.iter_mut().rev() {
                let next_overflow_from_above = overflow_from_below_iter.next().unwrap();
                let new_raw = next_overflow_from_above.raw | overflow_from_before.raw;
                overflow_from_before.raw = cell.raw & TO_WEST;
                next_overflow_from_above.raw = cell.raw & (TO_NORTH_WEST);
                current_overflow_from_above.raw |= cell.raw & (TO_NORTH_EAST);
                cell.raw = new_raw | (cell.raw & (TO_SOUTH_EAST | TO_SOUTH_WEST | TO_EAST));

                current_overflow_from_above = next_overflow_from_above;
            }
        } else {
            let mut temp: u8 = current_overflow_from_above.raw;
            for cell in row.iter_mut().rev() {
                let next_overflow_from_above = overflow_from_below_iter.next().unwrap();

                let new_raw = temp | overflow_from_before.raw;
                temp = next_overflow_from_above.raw;
                overflow_from_before.raw = cell.raw & TO_WEST;
                next_overflow_from_above.raw = cell.raw & (TO_NORTH_WEST);
                current_overflow_from_above.raw |= cell.raw & (TO_NORTH_EAST);
                cell.raw = new_raw | (cell.raw & (TO_SOUTH_EAST | TO_SOUTH_WEST | TO_EAST));

                current_overflow_from_above = next_overflow_from_above;
            }
        }
    }

    // let mut result = Box::new([Cell::new(); (WIDTH + 2) * (HEIGHT + 2)]);
    // // let mut row_refs: [&[Cell; WIDTH + 2]; HEIGHT + 2] = [&[]; HEIGHT + 2];
    // let mut mut_slices = result.array_chunks_mut::<{ WIDTH + 2 }>();

    // let mut previous_output_row = mut_slices.next().unwrap();
    // let mut current_output_row = mut_slices.next().unwrap();
    // let mut next_output_row = mut_slices.next().unwrap();
    // for input_row in section.array_chunks::<WIDTH>() {
    //     input_row.iter().enumerate().for_each(|(x, cell)| {
    //         current_output_row[x].receive_from_east(cell);
    //         previous_output_row[x].receive_from_south_east(cell);
    //         previous_output_row[x].receive_from_south_west(cell);
    //         current_output_row[x].receive_from_west(cell);
    //         next_output_row[x].receive_from_north_west(cell);
    //         next_output_row[x].receive_from_north_east(cell);
    //     });
    //     previous_output_row = current_output_row;
    //     current_output_row = next_output_row;
    //     next_output_row = mut_slices.next().unwrap();
    // }

    // &mut result.array_windows()

    // section.array_chunks::<WIDTH>().enumerate().for_each(
    //     |(y, row)| {
    //         let mut previous_row = &mut result.array_windows();
    //         let mut current_row = accumulator[(y + 1) * (WIDTH + 2)..(y + 2) * (WIDTH + 2)];
    //         let mut next_row = accumulator[(y + 2) * (WIDTH + 2)..(y + 3) * (WIDTH + 2)];
    //         row.iter().enumerate().for_each(|(x, cell)| {
    //             previous_row[x]
    //             cell.propagate();
    //             result[(y + 1) * (WIDTH + 2) + x + 1] = cell;
    //         });
    //         return accumulator;
    //     },

    // );
}

pub fn process_collisions<const WIDTH: usize, const HEIGHT: usize>(
    section: &mut [Cell; WIDTH * HEIGHT],
    seed: u32,
) where
    [(); WIDTH * HEIGHT]:,
{
    for cell in section.iter_mut() {
        cell.process_collision(seed);
    }
}

// pub fn propagate_section_core<const WIDTH: usize, const HEIGHT: usize>(
//     section: &[Cell; WIDTH * HEIGHT],
// ) -> Box<[Cell; (WIDTH + 2) * (HEIGHT + 2)]>
// where
//     [(); (WIDTH + 2) * (HEIGHT + 2)]:,
//     [(); WIDTH * HEIGHT]:,
//     [(); WIDTH + 2]:,
// {
//     let mut result = Box::new([Cell::new(), Cell::new(), Cell::new(), Cell::new()]);
//     // let mut row_refs: [&[Cell; WIDTH + 2]; HEIGHT + 2] = [&[]; HEIGHT + 2];
//     let mut mut_slices = result.array_chunks_mut::<{ WIDTH + 2 }>();

//     let mut previous_output_row = mut_slices.next().unwrap();
//     let mut current_output_row = mut_slices.next().unwrap();
//     let mut next_output_row = mut_slices.next().unwrap();
//     for input_row in section.array_chunks::<WIDTH>() {
//         input_row.iter().enumerate().for_each(|(x, cell)| {
//             current_output_row[x].receive_from_east(cell);
//             previous_output_row[x].receive_from_south_east(cell);
//             previous_output_row[x].receive_from_south_west(cell);
//             current_output_row[x].receive_from_west(cell);
//             next_output_row[x].receive_from_north_west(cell);
//             next_output_row[x].receive_from_north_east(cell);
//         });
//         previous_output_row = current_output_row;
//         current_output_row = next_output_row;
//         next_output_row = mut_slices.next().unwrap();
//     }

//     // &mut result.array_windows()

//     // section.array_chunks::<WIDTH>().enumerate().for_each(
//     //     |(y, row)| {
//     //         let mut previous_row = &mut result.array_windows();
//     //         let mut current_row = accumulator[(y + 1) * (WIDTH + 2)..(y + 2) * (WIDTH + 2)];
//     //         let mut next_row = accumulator[(y + 2) * (WIDTH + 2)..(y + 3) * (WIDTH + 2)];
//     //         row.iter().enumerate().for_each(|(x, cell)| {
//     //             previous_row[x]
//     //             cell.propagate();
//     //             result[(y + 1) * (WIDTH + 2) + x + 1] = cell;
//     //         });
//     //         return accumulator;
//     //     },

//     // );

//     todo!()
// }

// pub trait Sync

#[cfg(test)]
mod tests {

    use crate::lgca::{cell::Cell, get_out_propagation, print_section};

    use super::*;

    #[test]
    fn out_propagation_works_without_values() {
        // Create a sample section
        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        let result = get_out_propagation::<2, 2>(&section);
        assert_eq!(result.above[0].raw, 0);
        assert_eq!(result.above[1].raw, 0);
        assert_eq!(result.after[0].raw, 0);
        assert_eq!(result.after[1].raw, 0);
        assert_eq!(result.below[0].raw, 0);
        assert_eq!(result.below[1].raw, 0);
        assert_eq!(result.before[0].raw, 0);
        assert_eq!(result.before[1].raw, 0);
    }

    #[test]
    fn out_propagation_works_with_values() {
        // Create a sample section
        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_west(true);
        section[0].set_to_north_east(true);
        section[0].set_to_north_west(true);
        section[1].set_to_north_east(true);
        section[1].set_to_north_west(true);
        section[1].set_to_east(true);
        section[2].set_to_west(true);
        section[2].set_to_south_east(true);
        section[2].set_to_south_west(true);
        section[3].set_to_south_east(true);
        section[3].set_to_south_west(true);
        section[3].set_to_east(true);

        let result = get_out_propagation::<2, 2>(&section);
        assert_eq!(
            result.above[0].to_north_west(),
            true,
            "{:?}",
            result.above[0]
        );
        assert_eq!(
            result.above[1].to_north_west(),
            true,
            "{:?}",
            result.above[1]
        );
        assert_eq!(
            result.above[1].to_north_east(),
            true,
            "{:?}",
            result.above[1]
        );
        assert_eq!(
            result.above[2].to_north_east(),
            true,
            "{:?}",
            result.above[2]
        );
        assert_eq!(result.before[0].to_west(), true, "{:?}", result.before[0]);
        assert_eq!(result.before[1].to_west(), true, "{:?}", result.before[1]);
        assert_eq!(result.after[0].to_east(), true, "{:?}", result.after[0]);
        assert_eq!(result.after[1].to_east(), true, "{:?}", result.after[1]);
        assert_eq!(
            result.below[0].to_south_west(),
            true,
            "{:?}",
            result.below[0]
        );
        assert_eq!(
            result.below[1].to_south_west(),
            true,
            "{:?}",
            result.below[1]
        );
        assert_eq!(
            result.below[1].to_south_east(),
            true,
            "{:?}",
            result.below[1]
        );
        assert_eq!(
            result.below[2].to_south_east(),
            true,
            "{:?}",
            result.below[2]
        );
    }

    #[test]
    pub fn test_input_propagation_works_to_east() {
        // Create a sample section
        let empty_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_east(), true, "{:?}", section[1]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_east(), false, "{:?}", section[1]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[2].set_to_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_east(), true, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[3].set_to_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_east(), false, "{:?}", section[3]);

        // Create a sample section
        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.before[0].set_to_east(true);
        filled_border.before[1].set_to_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_east(true);

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_east(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_east(), true, "{:?}", section[1]);
        assert_eq!(section[2].to_east(), true, "{:?}", section[2]);
        assert_eq!(section[3].to_east(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_south_east() {
        // Create a sample section
        let empty_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_south_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), true, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_south_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[2].set_to_south_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[3].set_to_south_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_south_east_from_border() {
        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.above[0].set_to_south_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_south_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), false, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.above[1].set_to_south_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_south_east(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), false, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.above[2].set_to_south_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_south_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_east(), true, "{:?}", section[1]);
        assert_eq!(section[2].to_south_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_east(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_south_west() {
        // Create a sample section
        let empty_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_south_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), true, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_south_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), true, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[2].set_to_south_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[3].set_to_south_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_south_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_south_west_from_border() {
        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.above[0].set_to_south_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_south_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), false, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.above[1].set_to_south_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_south_west(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), false, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.above[2].set_to_south_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_south_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_south_west(), true, "{:?}", section[1]);
        assert_eq!(section[2].to_south_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_south_west(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_west() {
        // Create a sample section
        let empty_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_west(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[2].set_to_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[3].set_to_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_west(), true, "{:?}", section[2]);
        assert_eq!(section[3].to_west(), false, "{:?}", section[3]);

        // Create a sample section
        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.after[0].set_to_west(true);
        filled_border.after[1].set_to_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_west(true);

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_west(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_west(), true, "{:?}", section[1]);
        assert_eq!(section[2].to_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_west(), true, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_north_west() {
        // Create a sample section
        let empty_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_north_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_north_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[2].set_to_north_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[3].set_to_north_west(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_west(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_north_west_from_border() {
        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.below[0].set_to_north_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_north_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), true, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), false, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.below[1].set_to_north_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_north_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), true, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.below[2].set_to_north_west(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_north_west(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_west(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_west(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_west(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_north_east() {
        // Create a sample section
        let empty_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[0].set_to_north_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[1].set_to_north_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[2].set_to_north_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_east(), true, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), false, "{:?}", section[3]);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];
        section[3].set_to_north_east(true);

        get_in_propagation::<2, 2>(&mut section, &empty_border);
        assert_eq!(section[0].to_north_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), true, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), false, "{:?}", section[3]);
    }

    #[test]
    pub fn test_input_propagation_works_to_north_east_from_border() {
        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.below[0].set_to_north_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_north_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), true, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), false, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.below[1].set_to_north_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_north_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), true, "{:?}", section[3]);

        let mut filled_border = SectionContext {
            above: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            before: Box::new([Cell::new(), Cell::new()]),
            below: Box::new([Cell::new(), Cell::new(), Cell::new()]),
            after: Box::new([Cell::new(), Cell::new()]),
        };
        filled_border.below[2].set_to_north_east(true);

        let mut section: [Cell; 4] = [Cell::new(), Cell::new(), Cell::new(), Cell::new()];

        get_in_propagation::<2, 2>(&mut section, &filled_border);
        assert_eq!(section[0].to_north_east(), false, "{:?}", section[0]);
        assert_eq!(section[1].to_north_east(), false, "{:?}", section[1]);
        assert_eq!(section[2].to_north_east(), false, "{:?}", section[2]);
        assert_eq!(section[3].to_north_east(), false, "{:?}", section[3]);
    }

    #[test]
    fn propagate_output_works_with_weird_bug() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[7].set_to_east(true);
        section[7].set_to_west(true);
        section[12].set_to_north_west(true);
        section[12].set_to_south_west(true);
        section[14].set_to_north_east(true);
        section[14].set_to_south_east(true);

        print_section::<WIDTH, HEIGHT>(&section);

        let out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
        println!(
            "exit after: {:?}",
            out_prop
                .before
                .iter()
                .map(|c| c.get_particles())
                .collect::<Vec<_>>()
        );
        assert_eq!(out_prop.below[0].to_south_west(), true);
        assert_eq!(out_prop.before[2].to_north_west(), true);
    }

    #[test]
    fn no_particels_get_lost_without_collisions() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[5].set_to_east(true);
        section[5].set_to_north_east(true);
        section[5].set_to_north_west(true);
        section[5].set_to_south_east(true);
        section[5].set_to_south_west(true);
        section[5].set_to_west(true);

        for _ in 0..30 {
            let out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            let mut reversed_context = SectionContext {
                above: out_prop.below,
                before: out_prop.after,
                below: out_prop.above,
                after: out_prop.before,
            };

            // eprintln!("=============== {}", x);
            // println!(
            //     "exit above: {:?}",
            //     reversed_context
            //         .below
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "exit below: {:?}",
            //     reversed_context
            //         .above
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "exit before: {:?}",
            //     reversed_context
            //         .after
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "exit after: {:?}",
            //     reversed_context
            //         .before
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            reversed_context.above[4].raw |= reversed_context.above[0].raw;
            reversed_context.below[0].raw |= reversed_context.below[4].raw;
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &reversed_context);
            // print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                6
            );
        }
    }

    #[test]
    fn no_particels_get_lost_with_collisions() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[5].set_to_east(true);
        section[5].set_to_north_east(true);
        section[5].set_to_north_west(true);
        section[5].set_to_south_east(true);
        section[5].set_to_south_west(true);
        section[5].set_to_west(true);

        for round in 0..30 {
            let out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            let mut reversed_context = SectionContext {
                above: out_prop.below,
                before: out_prop.after,
                below: out_prop.above,
                after: out_prop.before,
            };

            // eprintln!("=============== {}", x);
            // println!(
            //     "exit above: {:?}",
            //     reversed_context
            //         .below
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "exit below: {:?}",
            //     reversed_context
            //         .above
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "exit before: {:?}",
            //     reversed_context
            //         .after
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            // println!(
            //     "exit after: {:?}",
            //     reversed_context
            //         .before
            //         .iter()
            //         .map(|c| c.get_particles())
            //         .collect::<Vec<_>>()
            // );
            reversed_context.above[4].raw |= reversed_context.above[0].raw;
            reversed_context.below[0].raw |= reversed_context.below[4].raw;
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &reversed_context);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            // print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                6
            );
        }
    }
}
