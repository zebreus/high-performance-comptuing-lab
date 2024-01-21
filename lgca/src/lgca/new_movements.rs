use super::{
    cell::{TO_EAST, TO_NORTH_EAST, TO_NORTH_WEST, TO_SOUTH_EAST, TO_SOUTH_WEST, TO_WEST},
    Cell,
};

/// Calculate the movements for the core of a section
#[inline(never)]
pub fn movement_core<const WIDTH: usize>(
    above: &[Cell; WIDTH - 1],
    current: &[Cell; WIDTH],
    below: &[Cell; WIDTH - 1],
    result: &mut [Cell; WIDTH - 2],
) {
    let context_iterator = above
        .array_windows::<2>()
        .zip(current.array_windows::<3>())
        .zip(below.array_windows::<2>())
        .zip(result.iter_mut());

    context_iterator.for_each(
        |(
            (([north_west, north_east], [west, _current, east]), [south_west, south_east]),
            result,
        )| {
            let mut temp = (west.raw & TO_EAST)
                | (north_west.raw & TO_SOUTH_EAST)
                | (north_east.raw & TO_SOUTH_WEST)
                | (east.raw & TO_WEST)
                | (south_east.raw & TO_NORTH_WEST)
                | (south_west.raw & TO_NORTH_EAST);
            // let mut temp = (west.raw & 0b00000001)
            //     | (north_west.raw & 0b00000010)
            //     | (north_east.raw & 0b00000100)
            //     | (east.raw & 0b00001000)
            //     | (south_east.raw & 0b00010000)
            //     | (south_west.raw & 0b00100000);
            result.raw = temp;
            // result.process_collision();
            if (temp == 0b00100100) && (south_east.raw & 0b00010000 == 0) {
                result.raw = 0b00010010;
            }
            if (temp == 0b00100100) && (south_east.raw & 0b00010000 != 0) {
                result.raw = 0b00001001;
            }
            if (temp == 0b00011011) && (south_east.raw & 0b00010000 == 0) {
                result.raw = 0b00101101;
            }
            if (temp == 0b00011011) && (south_east.raw & 0b00010000 != 0) {
                result.raw = 0b00110110;
            }

            if temp == 0b00010010 && (east.raw & 0b00001000 == 0) {
                result.raw = 0b00001001;
            }
            if temp == 0b00010010 && (east.raw & 0b00001000 != 0) {
                result.raw = 0b00100100;
            }
            if temp == 0b00101101 && (east.raw & 0b00001000 == 0) {
                result.raw = 0b00110110;
            }
            if temp == 0b00101101 && (east.raw & 0b00001000 != 0) {
                result.raw = 0b00011011;
            }

            if temp == 0b00001001 && (north_east.raw & 0b00000100 == 0) {
                result.raw = 0b00100100;
            }
            if temp == 0b00001001 && (north_east.raw & 0b00000100 != 0) {
                result.raw = 0b00010010;
            }
            if temp == 0b00110110 && (north_east.raw & 0b00000100 == 0) {
                result.raw = 0b00011011;
            }
            if temp == 0b00110110 && (north_east.raw & 0b00000100 != 0) {
                result.raw = 0b00101101;
            }

            if temp == 0b00101010 {
                result.raw = 0b00010101;
            }
            if temp == 0b00010101 {
                result.raw = 0b00101010;
            }
            temp = result.raw; // This line does nothing, but autovectorization breaks without it
        },
    )
}

/// Calculate the movement of the core of the top row
fn movement_core_top<const WIDTH: usize>(
    current: &[Cell; WIDTH],
    below: &[Cell; WIDTH - 1],
    result: &mut [Cell; WIDTH - 2],
) {
    let context_iterator = current
        .array_windows::<3>()
        .zip(below.array_windows::<2>())
        .zip(result.iter_mut());

    context_iterator.for_each(
        |(([west, current, east], [south_west, south_east]), result)| {
            result.raw = (west.raw & TO_EAST)
                | ((current.raw & TO_NORTH_EAST) << 2)
                | ((current.raw & TO_NORTH_WEST) << 4)
                | (east.raw & TO_WEST)
                | (south_east.raw & TO_NORTH_WEST)
                | (south_west.raw & TO_NORTH_EAST);
            result.process_collision();
        },
    )
}

/// Calculate the movement of the core of the bottom row
fn movement_core_bottom<const WIDTH: usize>(
    above: &[Cell; WIDTH - 1],
    current: &[Cell; WIDTH],
    result: &mut [Cell; WIDTH - 2],
) {
    let context_iterator = above
        .array_windows::<2>()
        .zip(current.array_windows::<3>())
        .zip(result.iter_mut());

    context_iterator.for_each(
        |(([north_west, north_east], [west, current, east]), result)| {
            result.raw = (west.raw & TO_EAST)
                | (north_west.raw & TO_SOUTH_EAST)
                | (north_east.raw & TO_SOUTH_WEST)
                | (east.raw & TO_WEST)
                | ((current.raw & TO_SOUTH_EAST) >> 2)
                | ((current.raw & TO_SOUTH_WEST) >> 4);
            result.process_collision();
        },
    )
}

pub fn movement_even_row<const WIDTH: usize>(
    above: &[Cell; WIDTH],
    current: &[Cell; WIDTH],
    below: &[Cell; WIDTH],
    result: &mut [Cell; WIDTH],
) where
    [(); WIDTH - 1]:,
    [(); WIDTH - 2]:,
{
    // Handle border of first cell
    result[0].raw = (above[0].raw & TO_SOUTH_EAST)
        | (above[1].raw & TO_SOUTH_WEST)
        | (below[0].raw & TO_NORTH_EAST)
        | (below[1].raw & TO_NORTH_WEST)
        | (current[1].raw & TO_WEST)
        | ((current[0].raw & TO_WEST) << 3);
    result[0].process_collision();

    // Handle core
    movement_core(
        above.rsplit_array_ref::<{ WIDTH - 1 }>().1,
        current,
        below.rsplit_array_ref::<{ WIDTH - 1 }>().1,
        result
            .rsplit_array_mut::<{ WIDTH - 1 }>()
            .1
            .split_array_mut::<{ WIDTH - 2 }>()
            .0,
    );

    // Handle border of last cell
    result[WIDTH - 1].raw = (above[WIDTH - 1].raw & TO_SOUTH_EAST)
        | (below[WIDTH - 1].raw & TO_NORTH_EAST)
        | (current[WIDTH - 2].raw & TO_EAST)
        | ((current[WIDTH - 1].raw & TO_EAST) >> 3)
        | ((current[WIDTH - 1].raw & TO_NORTH_EAST) >> 1)
        | ((current[WIDTH - 1].raw & TO_SOUTH_EAST) << 1);
    result[WIDTH - 1].process_collision();
}

/// Top row is always even
pub fn movement_top_row<const WIDTH: usize>(
    current: &[Cell; WIDTH],
    below: &[Cell; WIDTH],
    result: &mut [Cell; WIDTH],
) where
    [(); WIDTH - 1]:,
    [(); WIDTH - 2]:,
{
    // Handle border of first cell
    result[0].raw = (below[0].raw & TO_NORTH_EAST)
        | (below[1].raw & TO_NORTH_WEST)
        | (current[1].raw & TO_WEST)
        | ((current[0].raw & TO_NORTH_EAST) << 2)
        | ((current[0].raw & TO_NORTH_WEST) << 4)
        | ((current[0].raw & TO_WEST) << 3);
    result[0].process_collision();

    // Handle core
    movement_core_top(
        current,
        below.rsplit_array_ref::<{ WIDTH - 1 }>().1,
        result
            .rsplit_array_mut::<{ WIDTH - 1 }>()
            .1
            .split_array_mut::<{ WIDTH - 2 }>()
            .0,
    );

    // Handle border of last cell
    result[WIDTH - 1].raw = ((current[WIDTH - 1].raw & TO_NORTH_WEST) << 3)
        | (below[WIDTH - 1].raw & TO_NORTH_EAST)
        | (current[WIDTH - 2].raw & TO_EAST)
        | ((current[WIDTH - 1].raw & TO_EAST) >> 3)
        | ((current[WIDTH - 1].raw & TO_NORTH_EAST) << 3)
        | ((current[WIDTH - 1].raw & TO_SOUTH_EAST) >> 3);
    result[WIDTH - 1].process_collision();
}

pub fn movement_odd_row<const WIDTH: usize>(
    above: &[Cell; WIDTH],
    current: &[Cell; WIDTH],
    below: &[Cell; WIDTH],
    result: &mut [Cell; WIDTH],
) where
    [(); WIDTH - 1]:,
    [(); WIDTH - 2]:,
{
    // Handle border of first cell
    result[0].raw = (above[0].raw & TO_SOUTH_WEST)
        | (below[0].raw & TO_NORTH_WEST)
        | (current[1].raw & TO_WEST)
        | ((current[0].raw & TO_WEST) << 3)
        | ((current[0].raw & TO_NORTH_WEST) << 1)
        | ((current[0].raw & TO_SOUTH_WEST) >> 1);
    result[0].process_collision();

    // Handle core
    movement_core(
        above.split_array_ref::<{ WIDTH - 1 }>().0,
        current,
        below.split_array_ref::<{ WIDTH - 1 }>().0,
        result
            .rsplit_array_mut::<{ WIDTH - 1 }>()
            .1
            .split_array_mut::<{ WIDTH - 2 }>()
            .0,
    );

    // Handle border of last cell
    result[WIDTH - 1].raw = (above[WIDTH - 2].raw & TO_SOUTH_EAST)
        | (above[WIDTH - 1].raw & TO_SOUTH_WEST)
        | (below[WIDTH - 2].raw & TO_NORTH_EAST)
        | (below[WIDTH - 1].raw & TO_NORTH_WEST)
        | (current[WIDTH - 2].raw & TO_EAST)
        | ((current[WIDTH - 1].raw & TO_EAST) >> 3);
    result[WIDTH - 1].process_collision();
}

pub fn movement_bottom_row<const WIDTH: usize>(
    above: &[Cell; WIDTH],
    current: &[Cell; WIDTH],
    result: &mut [Cell; WIDTH],
) where
    [(); WIDTH - 1]:,
    [(); WIDTH - 2]:,
{
    // Handle border of first cell
    result[0].raw = (above[0].raw & TO_SOUTH_WEST)
        | (current[1].raw & TO_WEST)
        | ((current[0].raw & TO_SOUTH_EAST) >> 3)
        | ((current[0].raw & TO_WEST) << 3)
        | ((current[0].raw & TO_NORTH_WEST) << 3)
        | ((current[0].raw & TO_SOUTH_WEST) >> 3);
    result[0].process_collision();

    // Handle core
    movement_core_bottom(
        above.split_array_ref::<{ WIDTH - 1 }>().0,
        current,
        result
            .rsplit_array_mut::<{ WIDTH - 1 }>()
            .1
            .split_array_mut::<{ WIDTH - 2 }>()
            .0,
    );

    // Handle border of last cell
    result[WIDTH - 1].raw = (above[WIDTH - 2].raw & TO_SOUTH_EAST)
        | (above[WIDTH - 1].raw & TO_SOUTH_WEST)
        | (current[WIDTH - 2].raw & TO_EAST)
        | ((current[WIDTH - 1].raw & TO_EAST) >> 3)
        | ((current[WIDTH - 1].raw & TO_SOUTH_EAST) >> 2)
        | ((current[WIDTH - 1].raw & TO_SOUTH_WEST) >> 4);
    result[WIDTH - 1].process_collision();
}

pub fn print_section<const WIDTH: usize>(section: &[[Cell; WIDTH]])
where
    [(); WIDTH]:,
{
    for (index, row) in section.iter().enumerate() {
        if (index % 2) == 0 {
            for cell in row.iter() {
                eprint!(" ");
                eprint!("{:?}", cell.get_particles());
            }
        } else {
            for cell in row.iter() {
                eprint!("{:?}", cell.get_particles());
                eprint!(" ");
            }
        }
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn even_rows_on_east_and_west() {
        const WIDTH: usize = 10;
        // Create a sample section
        let above: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        current[WIDTH - 1].set_to_east(true);
        current[0].set_to_west(true);

        movement_even_row(&above, &current, &below, &mut section);

        assert_eq!(section[0].to_east(), true);
        assert_eq!(section[WIDTH - 1].to_west(), true);

        std::mem::swap(&mut current, &mut section);
        movement_even_row(&above, &current, &below, &mut section);

        assert_eq!(section[1].to_east(), true);
        assert_eq!(section[WIDTH - 2].to_west(), true);
    }

    #[test]
    fn move_down_into_even_rows() {
        const WIDTH: usize = 10;
        // Create a sample section
        let mut above: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        above[1].set_to_south_east(true);
        above[1].set_to_south_west(true);

        movement_even_row(&above, &current, &below, &mut section);

        assert_eq!(section[0].to_south_west(), true);
        assert_eq!(section[1].to_south_east(), true);
        assert_eq!(
            section
                .iter()
                .map(|c| c.get_particles() as usize)
                .sum::<usize>(),
            2
        );
    }

    #[test]
    fn move_to_corner_of_top_row_works() {
        const WIDTH: usize = 10;
        // Create a sample section
        let mut current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        current[0].set_to_north_west(true);
        current[1].set_to_north_east(true);

        movement_top_row(&current, &below, &mut section);

        assert_eq!(
            section
                .iter()
                .map(|c| c.get_particles() as usize)
                .sum::<usize>(),
            2
        );
        // eprintln!("{:?}", section[0]);
        // eprintln!("{:?}", section[1]);
        assert_eq!(section[0].to_south_east(), true);
        assert_eq!(section[1].to_south_east(), true);
    }

    #[test]
    fn odd_row_side_bounce_works() {
        const WIDTH: usize = 10;
        // Create a sample section
        let above: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        current[0].set_to_south_west(true);

        movement_odd_row(&above, &current, &below, &mut section);

        assert_eq!(
            section
                .iter()
                .map(|c| c.get_particles() as usize)
                .sum::<usize>(),
            1
        );
        eprintln!("{:?}", section[0]);
        assert_eq!(section[0].to_south_east(), true);
    }

    #[test]
    fn odd_row_side_bounce_works_b() {
        const WIDTH: usize = 10;
        // Create a sample section
        let above: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        current[0].set_to_north_west(true);

        movement_odd_row(&above, &current, &below, &mut section);

        assert_eq!(
            section
                .iter()
                .map(|c| c.get_particles() as usize)
                .sum::<usize>(),
            1
        );
        eprintln!("{:?}", section[0]);
        assert_eq!(section[0].to_north_east(), true);
    }

    #[test]
    fn even_row_side_bounce_works() {
        const WIDTH: usize = 10;
        // Create a sample section
        let above: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        current[WIDTH - 1].set_to_south_east(true);

        movement_even_row(&above, &current, &below, &mut section);

        assert_eq!(
            section
                .iter()
                .map(|c| c.get_particles() as usize)
                .sum::<usize>(),
            1
        );
        eprintln!("{:?}", section[WIDTH - 1]);
        assert_eq!(section[WIDTH - 1].to_south_west(), true);
    }

    #[test]
    fn even_row_side_bounce_works_b() {
        const WIDTH: usize = 10;
        // Create a sample section
        let above: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut current: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let below: [Cell; WIDTH] = [Cell::new(); WIDTH];
        let mut section: [Cell; WIDTH] = [Cell::new(); WIDTH];

        current[WIDTH - 1].set_to_north_east(true);

        movement_even_row(&above, &current, &below, &mut section);

        assert_eq!(
            section
                .iter()
                .map(|c| c.get_particles() as usize)
                .sum::<usize>(),
            1
        );
        eprintln!("{:?}", section[WIDTH - 1]);
        assert_eq!(section[WIDTH - 1].to_north_west(), true);
    }

    #[test]
    fn interactive_test() {
        const WIDTH: usize = 30;
        const HEIGHT: usize = 30;
        // Create a sample section
        let mut sections = [[Cell::new(); WIDTH]; HEIGHT];
        let mut sections_b = [[Cell::new(); WIDTH]; HEIGHT];

        sections[1][1].raw = 0b00111111;
        // sections[0][1].raw = TO_NORTH_WEST;
        eprintln!("============================ Intial");
        print_section(&sections[..]);
        for round in 0..50 {
            movement_top_row(&sections[0], &sections[1], &mut sections_b[0]);
            for (row, ([above, current, below], result)) in sections
                .array_windows::<3>()
                .zip(sections_b.iter_mut().skip(1))
                .enumerate()
            {
                if ((row + 1) % 2) == 0 {
                    movement_even_row(above, current, below, result);
                } else {
                    movement_odd_row(above, current, below, result);
                }
            }
            movement_bottom_row(
                &sections[WIDTH - 2],
                &sections[WIDTH - 1],
                &mut sections_b[WIDTH - 1],
            );

            for cell in sections_b.iter_mut().flatten() {
                cell.process_collision();
            }

            eprintln!("============================ Round {}", round);
            print_section(&sections_b[..]);
            std::mem::swap(&mut sections, &mut sections_b);
        }
        // assert_eq!(section[0].to_east(), true);
        // assert_eq!(section[WIDTH - 1].to_west(), true);

        // std::mem::swap(&mut current, &mut section);
        // movement_even_row(&above, &current, &below, &mut section);

        // assert_eq!(section[1].to_east(), true);
        // assert_eq!(section[WIDTH - 2].to_west(), true);
    }
}
