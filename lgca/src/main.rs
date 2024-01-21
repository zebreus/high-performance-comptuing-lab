#![feature(generic_const_exprs)]
#![feature(array_chunks)]
#![feature(iter_map_windows)]
#![feature(array_windows)]
#![feature(new_uninit)]
#![feature(split_array)]
#![feature(inline_const_pat)]

mod lgca;
use rand::prelude::*;

use clap::Parser;
use lgca::Cell;
use mpi::traits::*;
use ril::{Frame, Image, ImageSequence, Rgb, Rgba};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::lgca::{
    new_movements::{
        movement_bottom_row, movement_even_row, movement_odd_row, movement_top_row, print_section,
    },
    visualization::{draw_cells_b, draw_cells_c, draw_cells_detailed},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File containing the input data
    input: PathBuf,

    /// Where to write the output files. Must be a directory
    output_directory: PathBuf,
}

fn main() {
    // let cli = Cli::parse();

    // let input = cli.input;
    // let output_directory = cli.output_directory;

    // let mpi_version = mpi::environment::library_version();
    // let mpi_universe = if mpi_version.is_ok() {
    //     mpi::initialize_with_threading(mpi::Threading::Single)
    // } else {
    //     None
    // };

    // let rank = mpi_universe.as_ref().map_or(0, |o| o.0.world().rank());

    // let input_file_exists = input.is_file();
    // if rank == 0 && !input_file_exists {
    //     eprintln!("Input file {:?} does not exist or is not a file", input);
    //     std::process::exit(1);
    // }

    // let output_is_directory = output_directory.is_dir();
    // if rank == 0 && !output_is_directory {
    //     eprintln!("Creating output directory {:?}", output_directory);
    //     std::fs::create_dir_all(&output_directory).unwrap();
    // }

    // let before_processing = Instant::now();

    // // Content

    // let duration = before_processing.elapsed();

    // if rank == 0 {
    //     println!("{}", duration.as_secs_f64());
    // }

    const WIDTH: usize = 2000;
    const HEIGHT: usize = 2000;

    const BOX: usize = 600;
    // Create a sample section
    let mut sections_box = Box::new([[Cell::new(); WIDTH]; HEIGHT]);
    let mut sections_b_box = Box::new([[Cell::new(); WIDTH]; HEIGHT]);
    let mut sections = sections_box.as_mut();
    let mut sections_b = sections_b_box.as_mut();

    sections[1][1].raw = 0b00111111;
    for x in 0..BOX {
        for y in 0..BOX {
            sections[y][x].raw = 0b00111111;
        }
    }

    const NOISE: f64 = 0.04;

    eprintln!("Genrating background noise");
    let random = &mut rand::thread_rng();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if random.gen_bool(NOISE) {
                sections[y][x].set_to_east(true)
            }
            if random.gen_bool(NOISE) {
                sections[y][x].set_to_north_east(true)
            }
            if random.gen_bool(NOISE) {
                sections[y][x].set_to_north_west(true)
            }
            if random.gen_bool(NOISE) {
                sections[y][x].set_to_south_east(true)
            }
            if random.gen_bool(NOISE) {
                sections[y][x].set_to_south_west(true)
            }
            if random.gen_bool(NOISE) {
                sections[y][x].set_to_west(true)
            }
        }
    }
    eprintln!("Genrated background noise");

    // for x in 0..75 {
    //     sections[100][25 + x].raw = 0b00001000;
    //     sections[100][100 + x].raw = 0b00000001;
    // }

    let time_before = Instant::now();
    const ROUNDS: u32 = 100000;
    // sections[0][1].raw = TO_NORTH_WEST;
    eprintln!("============================ Intial");
    let mut images: Vec<Image<Rgb>> = Vec::new();
    images.push(draw_cells_detailed(sections).resized(
        (WIDTH / 4) as u32 + 3,
        (HEIGHT / 4) as u32 + 3,
        ril::ResizeAlgorithm::Lanczos3,
    ));
    for round in 0..ROUNDS {
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

        // for cell in sections_b.iter_mut().flatten() {
        //     cell.process_collision(round);
        // }

        if round % 20 == 0 {
            eprintln!("============================ Round {}", round);
            images.push(draw_cells_detailed(sections_b).resized(
                (WIDTH / 4) as u32 + 3,
                (HEIGHT / 4) as u32 + 3,
                ril::ResizeAlgorithm::Lanczos3,
            ));
        }

        std::mem::swap(&mut sections, &mut sections_b);
    }

    let duration = time_before.elapsed();
    eprintln!("Duration: {}", duration.as_secs_f64());
    eprintln!(
        "Duration per round: {}",
        duration.as_secs_f64() / ROUNDS as f64
    );

    let mut output = ImageSequence::<Rgb>::new();

    // ImageSequence::open is lazy
    for frame in images {
        let mut frame = Frame::from_image(frame);
        frame.set_delay(Duration::from_millis(15));
        output.push_frame(frame);
    }

    output.save_inferred("inverted.gif").unwrap();
}
