use super::Cell;

pub fn print_section<const WIDTH: usize, const HEIGHT: usize>(section: &[Cell; WIDTH * HEIGHT])
where
    [(); WIDTH * HEIGHT]:,
{
    for (index, row) in section.array_chunks::<WIDTH>().enumerate() {
        if (index % 2) == 0 {
            for cell in row {
                eprint!(" ");
                eprint!("{:?}", cell.get_particles());
            }
        } else {
            for cell in row {
                eprint!("{:?}", cell.get_particles());
                eprint!(" ");
            }
        }
        eprintln!();
    }
}
