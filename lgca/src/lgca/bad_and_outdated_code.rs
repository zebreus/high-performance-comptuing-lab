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

impl<const WIDTH: usize, const HEIGHT: usize> SectionContext<WIDTH, HEIGHT>
where
    [(); WIDTH + 1]:,
    [(); HEIGHT]:,
{
    /// Setup an exchange between this section and the section below it

    pub fn travel_down<const OTHER: usize>(&mut self, other: &mut SectionContext<WIDTH, OTHER>) {
        std::mem::swap(&mut other.above, &mut self.below);
        self.below[0].raw |= self.below[4].raw;
        other.above[4].raw |= other.above[0].raw;
    }
    /// Setup an exchange between this section and the section above it
    pub fn travel_up<const OTHER: usize>(&mut self, other: &mut SectionContext<WIDTH, OTHER>) {
        other.travel_down(self);
    }
    /// Setup an exchange between this section and the section right to it
    pub fn travel_right(&mut self, other: &mut SectionContext<WIDTH, HEIGHT>) {
        std::mem::swap(&mut other.before, &mut self.after);
    }
    /// Setup an exchange between this section and the section left to it
    pub fn travel_left(&mut self, other: &mut SectionContext<WIDTH, HEIGHT>) {
        std::mem::swap(&mut other.after, &mut self.before);
    }

    pub fn receive_from_above(&mut self, other: &[Cell; WIDTH + 1]) {
        self.above.copy_from_slice(other);
    }
    pub fn receive_from_below(&mut self, other: &[Cell; WIDTH + 1]) {
        self.below.copy_from_slice(other);
    }
    pub fn receive_from_left(&mut self, other: &[Cell; HEIGHT]) {
        self.before.copy_from_slice(other);
    }
    pub fn receive_from_right(&mut self, other: &[Cell; HEIGHT]) {
        self.after.copy_from_slice(other);
    }

    /// Process a top border. Does not process the top left and top right corner
    pub fn border_up(&mut self) {
        let mut cells_iter = self.above.iter_mut();
        let mut previous = cells_iter.next().unwrap();
        for cell in cells_iter {
            if (previous.raw & TO_NORTH_EAST) != 0 {
                cell.raw |= TO_SOUTH_EAST;
            }
            if (previous.raw & TO_NORTH_WEST) != 0 {
                previous.raw |= TO_SOUTH_WEST;
            }
            previous = cell;
        }
    }

    /// Reflect the output at the lower border of this section
    pub fn border_down(&mut self) {
        let mut cells_iter = self.below.iter_mut();
        let mut previous = cells_iter.next().unwrap();
        for cell in cells_iter {
            if (cell.raw & TO_SOUTH_EAST) != 0 {
                cell.raw |= TO_NORTH_EAST;
            }
            if (cell.raw & TO_SOUTH_WEST) != 0 {
                previous.raw |= TO_NORTH_WEST;
            }
            previous = cell;
        }
    }

    /// Process a left border. Does not process corners
    pub fn border_left(&mut self) {
        self.before[0].raw |= (self.before[0].raw & TO_WEST) << 3;

        let mut cells_iter = self.before.iter_mut();
        let mut previous = cells_iter.next().unwrap();

        for cell in cells_iter {
            cell.raw |= (cell.raw & TO_WEST) << 3;
            if (previous.raw & TO_SOUTH_WEST) != 0 {
                cell.raw |= TO_SOUTH_EAST;
            }
            if (cell.raw & TO_NORTH_WEST) != 0 {
                previous.raw |= TO_NORTH_EAST;
            }
            previous = cell;
        }
    }

    /// Process a right border. Does not process corners
    pub fn border_right(&mut self) {
        self.after[0].raw |= (self.after[0].raw & TO_EAST) >> 3;
        // self.after[0].raw |= (self.after[1].raw & TO_NORTH_EAST) << 3;

        let mut cells_iter = self.after.iter_mut();
        let mut previous = cells_iter.next().unwrap();

        for cell in cells_iter {
            cell.raw |= (cell.raw & TO_EAST) >> 3;
            if (previous.raw & TO_SOUTH_EAST) != 0 {
                cell.raw |= TO_SOUTH_WEST;
            }
            if (cell.raw & TO_NORTH_EAST) != 0 {
                previous.raw |= TO_NORTH_WEST;
            }
            previous = cell;
        }
    }

    /// Process the top right border, if there is a section to the right
    pub fn border_up_right(&self, next_right_above: &mut [Cell; WIDTH + 1]) {
        if (self.above[WIDTH].raw & TO_NORTH_EAST) != 0 {
            next_right_above[1].raw |= TO_SOUTH_EAST;
        }
    }

    /// Process the top right border, if there is a section to the right
    pub fn border_up_left(&self, next_left_above: &mut [Cell; WIDTH + 1]) {
        if (self.above[0].raw & TO_NORTH_WEST) != 0 {
            next_left_above[WIDTH].raw |= TO_SOUTH_EAST;
        }
    }

    /// Process the top right corner, if there is a section to the right
    pub fn corner_up_right(&mut self) {
        self.above[WIDTH].raw |= (self.above[WIDTH].raw & TO_NORTH_EAST) << 3;
    }
    /// Process the top right corner, if there is a section to the right
    pub fn corner_up_left(&mut self) {
        self.above[1].raw |= (self.above[0].raw & TO_NORTH_WEST) << 3;
        self.before[1].raw |= (self.before[0].raw & TO_NORTH_WEST) << 3;
    }

    /// Process the top right corner, if there is a section to the right
    pub fn corner_down_right(&mut self) {
        self.below[WIDTH - 1].raw |= (self.below[WIDTH].raw & TO_SOUTH_EAST) >> 3;
        self.after[HEIGHT - 2].raw |= (self.after[HEIGHT - 1].raw & TO_SOUTH_EAST) >> 3;
    }
    /// Process the top right corner, if there is a section to the right
    pub fn corner_down_left(&mut self) {
        self.below[0].raw |= (self.below[0].raw & TO_SOUTH_WEST) >> 3;
    }

    // /// Reflect the output at the top border of this section
    // pub fn border_up(
    //     &mut self,
    //     maybe_left: Option<&mut SectionContext<WIDTH, HEIGHT>>,
    //     maybe_right: Option<&mut SectionContext<WIDTH, HEIGHT>>,
    // ) {
    //     if (self.above[0].raw & TO_NORTH_WEST) != 0 {
    //         if let Some(left) = maybe_left {
    //             left.above[WIDTH].raw |= TO_SOUTH_WEST;
    //         } else {
    //             self.above[1].raw |= TO_SOUTH_EAST;
    //         }
    //     }
    //     let mut cells_iter = self.above.iter_mut().enumerate();
    //     let mut previous_previous = cells_iter.next().unwrap().1;
    //     let mut previous = cells_iter.next().unwrap().1;
    //     for (index, cell) in cells_iter {
    //         if (previous.raw & TO_NORTH_EAST) != 0 {
    //             cell.raw |= TO_SOUTH_EAST;
    //         }
    //         if (previous.raw & TO_NORTH_WEST) != 0 {
    //             previous.raw |= TO_SOUTH_WEST;
    //         }
    //         previous_previous = previous;
    //         previous = cell;
    //     }

    //     if (self.above[WIDTH].raw & TO_NORTH_EAST) != 0 {
    //         if let Some(right) = maybe_right {
    //             right.above[1].raw |= TO_SOUTH_EAST;
    //         } else {
    //             self.above[WIDTH].raw |= TO_SOUTH_WEST;
    //         }
    //     }
    // }

    // /// Reflect the output at the lower border of this section
    // pub fn border_down(
    //     &mut self,
    //     maybe_left: Option<&mut SectionContext<WIDTH, HEIGHT>>,
    //     maybe_right: Option<&mut SectionContext<WIDTH, HEIGHT>>,
    // ) {
    //     if (self.below[0].raw & TO_SOUTH_WEST) != 0 {
    //         if let Some(left) = maybe_left {
    //             left.below[WIDTH - 1].raw |= TO_NORTH_WEST;
    //         } else {
    //             self.below[0].raw |= TO_NORTH_EAST;
    //         }
    //     }
    //     let mut cells_iter = self.below.iter_mut().enumerate();
    //     let mut previous = cells_iter.next().unwrap().1;
    //     for (index, cell) in cells_iter {
    //         if (cell.raw & TO_SOUTH_EAST) != 0 {
    //             cell.raw |= TO_NORTH_EAST;
    //         }
    //         if (cell.raw & TO_SOUTH_WEST) != 0 {
    //             previous.raw |= TO_NORTH_WEST;
    //         }
    //         previous = cell;
    //     }

    //     if (self.below[WIDTH].raw & TO_SOUTH_EAST) != 0 {
    //         if let Some(right) = maybe_right {
    //             right.below[0].raw |= TO_NORTH_EAST;
    //         } else {
    //             self.below[WIDTH - 1].raw |= TO_NORTH_WEST;
    //         }
    //     }

    //     for cell in self.below.iter_mut() {
    //         cell.raw &= !(TO_SOUTH_EAST | TO_SOUTH_WEST);
    //     }
    // }
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

struct AssertEvenHeight<const HEIGHT: usize>;

impl<const HEIGHT: usize> AssertEvenHeight<HEIGHT> {
    const OK: () = assert!(HEIGHT % 2 == 0, "HEIGHT must be even");
}

pub fn get_out_propagation<const WIDTH: usize, const HEIGHT: usize>(
    section: &[Cell; WIDTH * HEIGHT],
) -> SectionContext<WIDTH, HEIGHT>
where
    [(); WIDTH + 1]:,
    [(); HEIGHT]:,
{
    let () = AssertEvenHeight::<HEIGHT>::OK;
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

// pub fn get_out_propagation_sections<
//     const WIDTH: usize,
//     const HEIGHT: usize,
//     const FULL_WIDTH: usize,
//     const FULL_HEIGHT: usize,
// >(
//     sections: &mut [&mut [Cell; WIDTH * HEIGHT]; 4],
// ) -> SectionContext<FULL_WIDTH, FULL_HEIGHT>
// where
//     [(); WIDTH * HEIGHT]:,
//     [(); FULL_WIDTH * FULL_HEIGHT]:,
//     [(); WIDTH + 1]:,
//     [(); FULL_WIDTH + 1]:,
//     [(); HEIGHT]:,
//     [(); WIDTH]:,
// {
//     let context_a = get_out_propagation(sections[0]);
//     let context_b = get_out_propagation(sections[1]);
//     let context_c = get_out_propagation(sections[2]);
//     let context_d = get_out_propagation(sections[3]);

//     let full_context = SectionContext {
//         above: Box::new([Cell::new(); FULL_WIDTH + 1]),
//         before: Box::new([Cell::new(); FULL_HEIGHT]),
//         below: Box::new([Cell::new(); FULL_WIDTH + 1]),
//         after: Box::new([Cell::new(); FULL_HEIGHT]),
//     };

//     full_context.above[0..(WIDTH + 1)].copy_from_slice(&context_a.above[..]);

//     let in_context_a = SectionContext {
//         above: Box::new(context.above[0..WIDTH].try_into().unwrap()),
//         before: context.before,
//         below: context.below,
//         after: context.after,
//     };

//     todo!();
// }

// pub fn process_partial_sections<
//     const WIDTH: usize,
//     const HEIGHT: usize,
//     const FULL_WIDTH: usize,
//     const FULL_HEIGHT: usize,
// >(
//     sections: &mut [&mut [Cell; WIDTH * HEIGHT]; 4],
//     context: &SectionContext<FULL_WIDTH, FULL_HEIGHT>,
// ) -> SectionContext<FULL_WIDTH, FULL_HEIGHT>
// where
//     [(); WIDTH * HEIGHT]:,
//     [(); FULL_WIDTH * FULL_HEIGHT]:,
//     [(); WIDTH + 1]:,
//     [(); FULL_WIDTH + 1]:,
//     [(); HEIGHT]:,
//     [(); WIDTH]:,
// {
//     let mut context_a = get_out_propagation(sections[0]);
//     let mut context_b = get_out_propagation(sections[1]);
//     let mut context_c = get_out_propagation(sections[2]);
//     let mut context_d = get_out_propagation(sections[3]);

//     context_a.receive_from_above(context.above.split_array_ref::<{ WIDTH + 1 }>().0);
//     context_b.receive_from_above(context.above.rsplit_array_ref::<{ WIDTH + 1 }>().1);

//     context_c.receive_from_below(context.below.split_array_ref::<{ WIDTH + 1 }>().0);
//     context_d.receive_from_below(context.below.rsplit_array_ref::<{ WIDTH + 1 }>().1);

//     context_a.receive_from_left(context.before.split_array_ref::<{ HEIGHT }>().0);
//     context_c.receive_from_left(context.before.rsplit_array_ref::<{ HEIGHT }>().1);

//     context_b.receive_from_right(context.after.split_array_ref::<{ HEIGHT }>().0);
//     context_d.receive_from_right(context.after.rsplit_array_ref::<{ HEIGHT }>().1);

//     context_a.travel_right(&mut context_b);
//     context_a.travel_down(&mut context_c);
//     context_b.travel_down(&mut context_d);
//     context_c.travel_right(&mut context_d);

//     //TODO: Threeway corner missing

//     cont

//     todo!();
// }

pub fn process_stacked_sections<const WIDTH: usize, const HEIGHT: usize, const FULL_HEIGHT: usize>(
    sections: &mut [&mut [Cell; WIDTH * HEIGHT]; 2],
    context: &mut SectionContext<WIDTH, FULL_HEIGHT>,
) where
    [(); WIDTH * HEIGHT]:,
    [(); WIDTH * FULL_HEIGHT]:,
    [(); WIDTH + 1]:,
    [(); HEIGHT]:,
    [(); WIDTH]:,
{
    let mut context_a = get_out_propagation(sections[0]);
    let mut context_b = get_out_propagation(sections[1]);

    context_a.travel_up(context);
    context_a.travel_down(&mut context_b);
    context_b.travel_down(context);

    std::mem::swap(
        context_a.before.split_array_mut::<{ HEIGHT }>().0,
        context.before.split_array_mut::<{ HEIGHT }>().0,
    );
    std::mem::swap(
        context_a.after.split_array_mut::<{ HEIGHT }>().0,
        context.after.split_array_mut::<{ HEIGHT }>().0,
    );
    std::mem::swap(
        context_b.before.split_array_mut::<{ HEIGHT }>().0,
        context.before.rsplit_array_mut::<{ HEIGHT }>().1,
    );
    std::mem::swap(
        context_b.after.split_array_mut::<{ HEIGHT }>().0,
        context.after.rsplit_array_mut::<{ HEIGHT }>().1,
    );

    //TODO: Threeway corner missing
    get_in_propagation(sections[0], &context_a);
    get_in_propagation(sections[1], &context_b);

    process_collisions(sections[0], 0);
    process_collisions(sections[1], 0);
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
                let new_raw = (next_overflow_from_above.raw & (TO_SOUTH_EAST | TO_SOUTH_WEST))
                    | (overflow_from_before.raw & (TO_EAST | TO_SOUTH_EAST));
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

                let new_raw = (temp & (TO_SOUTH_EAST | TO_SOUTH_WEST))
                    | (overflow_from_before.raw & (TO_EAST | TO_SOUTH_EAST));
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

    let rows_iterator = section
        .array_chunks_mut::<WIDTH>()
        .zip(after.iter())
        .zip(before.iter());
    for (row_index, ((row, after), before)) in rows_iterator.enumerate().rev() {
        let mut overflow_from_below_iter = overflow_from_below_row.iter_mut().rev();
        let mut current_overflow_from_above = overflow_from_below_iter.next().unwrap();
        let mut overflow_from_before = after.clone();
        if row_index % 2 == 1 {
            current_overflow_from_above.raw = 0;
            for cell in row.iter_mut().rev() {
                let next_overflow_from_below = overflow_from_below_iter.next().unwrap();
                let new_raw = (next_overflow_from_below.raw & (TO_NORTH_EAST | TO_NORTH_WEST))
                    | (overflow_from_before.raw & (TO_WEST | TO_NORTH_WEST | TO_SOUTH_WEST));
                overflow_from_before.raw = cell.raw & TO_WEST;
                next_overflow_from_below.raw = cell.raw & (TO_NORTH_WEST);
                current_overflow_from_above.raw |= cell.raw & (TO_NORTH_EAST);
                cell.raw = new_raw | (cell.raw & (TO_SOUTH_EAST | TO_SOUTH_WEST | TO_EAST));

                current_overflow_from_above = next_overflow_from_below;
            }
        } else {
            let mut temp: u8 = current_overflow_from_above.raw;
            for cell in row.iter_mut().rev() {
                let next_overflow_from_below = overflow_from_below_iter.next().unwrap();

                let new_raw = (temp & (TO_NORTH_EAST | TO_NORTH_WEST))
                    | (overflow_from_before.raw & (TO_WEST | TO_NORTH_WEST | TO_SOUTH_WEST));
                temp = next_overflow_from_below.raw;
                overflow_from_before.raw = cell.raw & TO_WEST;
                next_overflow_from_below.raw = cell.raw & (TO_NORTH_WEST);
                current_overflow_from_above.raw |= cell.raw & (TO_NORTH_EAST);
                cell.raw = new_raw | (cell.raw & (TO_SOUTH_EAST | TO_SOUTH_WEST | TO_EAST));

                current_overflow_from_above = next_overflow_from_below;
            }
        }

        // eprintln!("row_index: {}", row_index);
        // eprintln!("before {}:", before);
        // row[WIDTH - 1].raw |= before.raw;
        row[0].raw |= before.raw & TO_NORTH_EAST;
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
    fn travel_functions_do_not_lose_particles() {
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
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            let mut reversed_context = SectionContext {
                above: unsafe { Box::new_zeroed().assume_init() },
                before: unsafe { Box::new_zeroed().assume_init() },
                below: unsafe { Box::new_zeroed().assume_init() },
                after: unsafe { Box::new_zeroed().assume_init() },
            };
            out_prop.travel_down(&mut reversed_context);
            out_prop.travel_up(&mut reversed_context);
            out_prop.travel_left(&mut reversed_context);
            out_prop.travel_right(&mut reversed_context);
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &reversed_context);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            // eprintln!("=============== {}", round);
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
    fn border_up_does_not_lose_particles() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[6].set_to_north_west(true);
        section[6].set_to_north_east(true);

        for round in 0..2 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_up();
            out_prop.corner_up_left();
            out_prop.corner_up_right();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            // eprintln!("=============== {}", round);
            // print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                2
            );
        }

        assert_eq!(section[0].get_particles(), 1);
        assert_eq!(section[3].get_particles(), 1);
    }

    #[test]
    fn border_up_works_on_corners() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[0].set_to_north_west(true);
        section[3].set_to_north_east(true);

        for round in 0..2 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_up();
            out_prop.corner_up_left();
            out_prop.corner_up_right();

            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);

            // eprintln!("=============== {}", round);
            // print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                2
            );
        }
        assert_eq!(section[5].get_particles(), 1);
        assert_eq!(section[7].get_particles(), 1);
    }

    #[test]
    fn border_down_works_on_edges() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[13].set_to_south_west(true);
        section[14].set_to_south_east(true);

        for round in 0..1 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_down();
            out_prop.corner_down_left();
            out_prop.corner_down_right();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            // eprintln!("=============== {}", round);
            // print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                2
            );
        }
        assert_eq!(section[12].get_particles(), 1);
        assert_eq!(section[15].get_particles(), 1);
    }

    #[test]
    fn border_down_works_on_corners() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[12].set_to_south_west(true);
        section[15].set_to_south_east(true);

        for round in 0..2 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_down();
            out_prop.corner_down_left();
            out_prop.corner_down_right();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            // eprintln!("=============== {}", round);
            // print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                2
            );
        }
        assert_eq!(section[8].get_particles(), 1);
        assert_eq!(section[10].get_particles(), 1);
    }

    #[test]
    fn border_left_works_on_edges() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[0].set_to_west(true);
        section[4].set_to_west(true);
        section[4].set_to_south_west(true);
        section[8].set_to_west(true);
        section[12].set_to_west(true);
        section[12].set_to_north_west(true);

        for round in 0..1 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_left();
            // out_prop.corner_down_left();
            // out_prop.corner_up_left();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            eprintln!("=============== {}", round);
            print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                6
            );
        }
        assert_eq!(section[0].get_particles(), 1);
        assert_eq!(section[4].get_particles(), 2);
        assert_eq!(section[8].get_particles(), 1);
        assert_eq!(section[12].get_particles(), 2);
    }

    // #[test]
    // fn border_left_works_on_edges_b() {
    //     const WIDTH: usize = 4;
    //     const HEIGHT: usize = 4;

    //     let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];

    //     section[12].set_to_north_west(true);
    //     print_section::<WIDTH, HEIGHT>(&section);

    //     for round in 0..1 {
    //         let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
    //         // eprintln!("=============== {}", x);
    //         println!(
    //             "exit above: {:?}",
    //             out_prop
    //                 .above
    //                 .iter()
    //                 .map(|c| c.get_particles())
    //                 .collect::<Vec<_>>()
    //         );
    //         println!(
    //             "exit below: {:?}",
    //             out_prop
    //                 .below
    //                 .iter()
    //                 .map(|c| c.get_particles())
    //                 .collect::<Vec<_>>()
    //         );
    //         println!(
    //             "exit before: {:?}",
    //             out_prop
    //                 .before
    //                 .iter()
    //                 .map(|c| c.get_particles())
    //                 .collect::<Vec<_>>()
    //         );
    //         println!(
    //             "exit after: {:?}",
    //             out_prop
    //                 .after
    //                 .iter()
    //                 .map(|c| c.get_particles())
    //                 .collect::<Vec<_>>()
    //         );
    //         out_prop.border_left();
    //         // out_prop.corner_down_left();
    //         // out_prop.corner_up_left();
    //         get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
    //         // process_collisions::<WIDTH, HEIGHT>(&mut section, round);
    //         eprintln!("=============== {}", round);
    //         print_section::<WIDTH, HEIGHT>(&section);
    //         assert_eq!(
    //             section
    //                 .iter()
    //                 .map(|a| a.get_particles())
    //                 .reduce(|a, b| a + b)
    //                 .unwrap(),
    //             1
    //         );
    //     }
    //     assert_eq!(section[0].get_particles(), 1);
    //     assert_eq!(section[4].get_particles(), 2);
    //     assert_eq!(section[8].get_particles(), 1);
    //     assert_eq!(section[12].get_particles(), 2);
    // }

    #[test]
    fn border_left_works_on_corners() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[0].set_to_north_west(true);
        section[4].set_to_north_west(true);
        section[12].set_to_south_west(true);

        for round in 0..1 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            // out_prop.border_left();
            out_prop.corner_down_left();
            out_prop.corner_up_left();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            eprintln!("=============== {}", round);
            print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                3
            );
        }
        assert_eq!(section[0].get_particles(), 1);
        assert_eq!(section[4].get_particles(), 1);
        assert_eq!(section[12].get_particles(), 1);
    }

    #[test]
    fn border_right_works_on_edges() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[3].set_to_east(true);
        section[3].set_to_south_east(true);
        section[7].set_to_east(true);
        section[11].set_to_east(true);
        section[11].set_to_north_east(true);
        section[15].set_to_east(true);

        for round in 0..1 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_right();
            // out_prop.corner_down_left();
            // out_prop.corner_up_left();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            eprintln!("=============== {}", round);
            print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                6
            );
        }
        assert_eq!(section[3].get_particles(), 2);
        assert_eq!(section[7].get_particles(), 1);
        assert_eq!(section[11].get_particles(), 2);
        assert_eq!(section[15].get_particles(), 1);
    }

    #[test]
    fn border_right_works_on_corners() {
        const WIDTH: usize = 4;
        const HEIGHT: usize = 4;

        let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
        section[3].set_to_north_east(true);
        section[11].set_to_south_east(true);
        section[15].set_to_south_east(true);

        for round in 0..1 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            // out_prop.border_left();
            out_prop.corner_down_right();
            out_prop.corner_up_right();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            eprintln!("=============== {}", round);
            print_section::<WIDTH, HEIGHT>(&section);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles())
                    .reduce(|a, b| a + b)
                    .unwrap(),
                3
            );
        }
        assert_eq!(section[3].get_particles(), 1);
        assert_eq!(section[11].get_particles(), 1);
        assert_eq!(section[15].get_particles(), 1);
    }

    #[test]
    fn no_particels_get_lost_with_collisions() {
        const WIDTH: usize = 10;
        const HEIGHT: usize = 10;

        let mut section: Box<[Cell; WIDTH * HEIGHT]> = Box::new([Cell::new(); WIDTH * HEIGHT]);
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

            reversed_context.above[4].raw |= reversed_context.above[0].raw;
            reversed_context.below[0].raw |= reversed_context.below[4].raw;
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &reversed_context);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);

            eprintln!("=============== {}", round);
            print_section::<WIDTH, HEIGHT>(&section);

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
    fn full_bordered_grid_does_not_lose_particles() {
        const WIDTH: usize = 8;
        const HEIGHT: usize = 8;

        let mut full_cell = Cell::new();
        full_cell.set_to_east(true);
        full_cell.set_to_north_east(true);
        full_cell.set_to_north_west(true);
        full_cell.set_to_south_east(true);
        full_cell.set_to_south_west(true);
        full_cell.set_to_west(true);
        let mut section: [Cell; WIDTH * HEIGHT] = [full_cell; WIDTH * HEIGHT];

        for round in 0..30 {
            let mut out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);
            out_prop.border_down();
            out_prop.border_up();
            out_prop.border_left();
            out_prop.border_right();
            out_prop.corner_down_left();
            out_prop.corner_down_right();
            out_prop.corner_up_left();
            out_prop.corner_up_right();
            get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
            process_collisions::<WIDTH, HEIGHT>(&mut section, round);
            assert_eq!(
                section
                    .iter()
                    .map(|a| a.get_particles() as usize)
                    .reduce(|a, b| a + b)
                    .unwrap(),
                WIDTH * HEIGHT * 6
            );
        }
    }

    // #[test]
    // fn interactive() {
    //     const WIDTH: usize = 8;
    //     const HEIGHT: usize = 8;

    //     let mut section: [Cell; WIDTH * HEIGHT] = [Cell::new(); WIDTH * HEIGHT];
    //     section[9].set_to_east(true);
    //     section[9].set_to_north_east(true);
    //     section[9].set_to_north_west(true);
    //     section[9].set_to_south_east(true);
    //     section[9].set_to_south_west(true);
    //     section[9].set_to_west(true);

    //     let mut partial_sections: [[Cell; WIDTH * (HEIGHT / 2)]; 2] =
    //         [[Cell::new(); WIDTH * (HEIGHT / 2)]; 2];
    //     partial_sections[0][9].set_to_east(true);
    //     partial_sections[0][9].set_to_north_east(true);
    //     partial_sections[0][9].set_to_north_west(true);
    //     partial_sections[0][9].set_to_south_east(true);
    //     partial_sections[0][9].set_to_south_west(true);
    //     partial_sections[0][9].set_to_west(true);

    //     for round in 0..30 {
    //         let mut normal_out_prop = get_out_propagation::<WIDTH, HEIGHT>(&section);

    //         normal_out_prop.border_down();
    //         normal_out_prop.border_up();
    //         normal_out_prop.border_left();
    //         normal_out_prop.border_right();
    //         normal_out_prop.corner_down_left();
    //         normal_out_prop.corner_down_right();
    //         normal_out_prop.corner_up_left();
    //         normal_out_prop.corner_up_right();

    //         get_in_propagation::<WIDTH, HEIGHT>(&mut section, &normal_out_prop);
    //         process_collisions::<WIDTH, HEIGHT>(&mut section, round);

    //         let

    //         // eprintln!("=============== {}", x);
    //         // println!(
    //         //     "exit above: {:?}",
    //         //     reversed_context
    //         //         .below
    //         //         .iter()
    //         //         .map(|c| c.get_particles())
    //         //         .collect::<Vec<_>>()
    //         // );
    //         // println!(
    //         //     "exit below: {:?}",
    //         //     reversed_context
    //         //         .above
    //         //         .iter()
    //         //         .map(|c| c.get_particles())
    //         //         .collect::<Vec<_>>()
    //         // );
    //         // println!(
    //         //     "exit before: {:?}",
    //         //     reversed_context
    //         //         .after
    //         //         .iter()
    //         //         .map(|c| c.get_particles())
    //         //         .collect::<Vec<_>>()
    //         // );
    //         // println!(
    //         //     "exit after: {:?}",
    //         //     reversed_context
    //         //         .before
    //         //         .iter()
    //         //         .map(|c| c.get_particles())
    //         //         .collect::<Vec<_>>()
    //         // );

    //         get_in_propagation::<WIDTH, HEIGHT>(&mut section, &out_prop);
    //         // process_collisions::<WIDTH, HEIGHT>(&mut section, round);
    //         eprintln!("=============== {}", round);
    //         print_section::<WIDTH, HEIGHT>(&section);
    //         assert_eq!(
    //             section
    //                 .iter()
    //                 .map(|a| a.get_particles())
    //                 .reduce(|a, b| a + b)
    //                 .unwrap(),
    //             6
    //         );
    //     }
    // }
}
