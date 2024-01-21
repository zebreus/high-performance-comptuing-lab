use ril::{Image, Rgb, TrueColor};

use super::Cell;

pub fn get_direction_of_cells(cells: &[&Cell]) -> (f32, f32) {
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    for cell in cells {
        if cell.to_east() {
            x += 1.0;
        }
        if cell.to_south_east() {
            x += 0.5;
            y += 0.866;
        }
        if cell.to_north_east() {
            x += 0.5;
            y -= 0.866;
        }

        if cell.to_west() {
            x -= 1.0;
        }
        if cell.to_south_west() {
            x -= 0.5;
            y += 0.866;
        }
        if cell.to_north_west() {
            x -= 0.5;
            y -= 0.866;
        }
    }

    let angle = y.atan2(x) / std::f32::consts::PI * 2.0;
    let length = (x * x + y * y).sqrt();

    return (angle, length);
}

pub fn get_particles_of_cells(cells: &[&Cell]) -> u32 {
    cells.iter().map(|cell| cell.get_particles() as u32).sum()
}

pub fn cells_to_color(cells: &[&Cell]) -> Rgb {
    let (angle, length) = get_direction_of_cells(cells);
    let particles = get_particles_of_cells(cells);
    let density = particles as f64 / (cells.len() * 6) as f64;
    // let lightness = match particles {
    //     0 => 0.0,
    //     1 => 0.5,
    //     2 => 1.0,
    //     _ => 0.0,
    // };

    let color = hsv::hsv_to_rgb(
        ((angle * 90.0) + 180.0) as f64,
        (2.0 * length as f64 / cells.len() as f64).min(1.0),
        (density * 6.0).min(1.0),
    );

    Rgb::from_rgb_tuple(color)
}

pub fn cell_to_color(cell: &Cell) -> Rgb {
    let (angle, length) = cell.get_direction();
    let particles = cell.get_particles();

    use hsl::HSL;

    let color = HSL {
        h: (angle * 90.0) as f64,
        s: length.min(1.0) as f64,
        l: if particles == 0 {
            0.0
        } else {
            (particles as f64 / 12.0) + 0.5
        },
    };

    Rgb::from_rgb_tuple(color.to_rgb())
}

pub fn draw_cells_detailed<const WIDTH: usize>(cells: &[[Cell; WIDTH]]) -> Image<Rgb> {
    let mut image = Image::new(WIDTH as u32, cells.len() as u32, Rgb::black());

    for (y, row) in cells.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            image.set_pixel(x as u32, y as u32, cell_to_color(pixel));
        }
    }
    return image;
}

#[allow(dead_code)]
pub fn draw_cells_b<const WIDTH: usize>(cells: &[[Cell; WIDTH]]) -> Image<Rgb> {
    let mut image = Image::new(cells.len() as u32, WIDTH as u32, Rgb::black());

    for (y, row) in cells.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            image.set_pixel(x as u32, y as u32, cells_to_color(&[pixel]));
        }
    }
    return image;
}

#[allow(dead_code)]
pub fn draw_cells_c<const WIDTH: usize>(cells: &[[Cell; WIDTH]]) -> Image<Rgb> {
    let mut image = Image::new(cells.len() as u32 / 4, WIDTH as u32 / 4, Rgb::black());

    for (y, rows) in cells.array_chunks::<4>().enumerate() {
        let iter = rows[0]
            .array_chunks::<4>()
            .zip(rows[1].array_chunks::<4>())
            .zip(rows[2].array_chunks::<4>())
            .zip(rows[3].array_chunks::<4>())
            .enumerate();
        for (x, (((a, b), c), d)) in iter {
            let cells = [
                &a[0], &a[1], &a[2], &a[3], &b[0], &b[1], &b[2], &b[3], &c[0], &c[1], &c[2], &c[3],
                &d[0], &d[1], &d[2], &d[3],
            ];
            image.set_pixel(x as u32, y as u32, cells_to_color(&cells));
        }
    }

    return image;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_iamges() {
        let mut cells = [[Cell::new(); 6]; 3];
        cells[0][0].set_to_west(true);
        cells[0][1].set_to_north_west(true);
        cells[0][2].set_to_north_east(true);
        cells[0][3].set_to_east(true);
        cells[0][4].set_to_south_east(true);
        cells[0][5].set_to_south_west(true);

        cells[1][0].set_to_west(true);
        cells[1][0].set_to_north_west(true);
        cells[1][1].set_to_north_west(true);
        cells[1][1].set_to_north_east(true);
        cells[1][2].set_to_north_east(true);
        cells[1][2].set_to_east(true);
        cells[1][3].set_to_east(true);
        cells[1][3].set_to_south_east(true);
        cells[1][4].set_to_south_east(true);
        cells[1][4].set_to_south_west(true);
        cells[1][5].set_to_south_west(true);
        cells[1][5].set_to_west(true);

        cells[2][1].set_to_west(true);
        cells[2][2].set_to_north_west(true);
        cells[2][3].set_to_north_east(true);
        cells[2][4].set_to_east(true);
        cells[2][5].set_to_south_east(true);
        cells[2][0].set_to_south_west(true);

        draw_cells_detailed(&cells)
            .save_inferred("sample_on_black.png")
            .unwrap();
    }
}
