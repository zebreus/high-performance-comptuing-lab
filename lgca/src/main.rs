#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_chunks)]
#![feature(iter_map_windows)]
#![feature(array_windows)]
#![feature(new_uninit)]
#![feature(split_array)]
#![feature(stmt_expr_attributes)]
#![feature(inline_const_pat)]

mod lgca;
use crate::lgca::{
    cell::{TO_EAST, TO_NORTH_EAST, TO_NORTH_WEST, TO_SOUTH_EAST, TO_SOUTH_WEST, TO_WEST},
    new_movements::{movement_bottom_row, movement_even_row, movement_odd_row, movement_top_row},
    visualization::draw_cells_detailed,
};
use clap::Parser;
use lgca::Cell;
use mpi::request::WaitGuard;
use mpi::topology::SimpleCommunicator;
use mpi::traits::*;
use rand::prelude::*;
use rayon::prelude::*;
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefMutIterator},
    slice::ParallelSlice,
};
use ril::{
    encodings::webp::{WebPEncoderOptions, WebPMuxEncoder},
    Encoder, EncoderMetadata, Frame, Image, ImageSequence, Rgb,
};
use std::{
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Where to write the output files. Must be a directory
    #[arg(short, long, default_value = "./")]
    output_directory: PathBuf,
    /// Percentage of particles filled with random noise
    #[arg(short, long, default_value_t = 0.04)]
    noise: f64,

    /// Percentage of particles filled with random noise
    #[arg(short, long, default_value_t = 5000)]
    rounds: usize,

    /// Speed in rounds per second
    #[arg(short, long, default_value_t = 1000)]
    speed: usize,

    /// Framerate in frames per second
    #[arg(short, long, default_value_t = 60)]
    framerate: usize,

    /// Scaling factor of the output video
    #[arg(long, default_value_t = 0.26)]
    scaling: f64,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 1)]
    threads: usize,

    /// Number of rows (Divided by number of mpi ranks)
    #[arg(long, default_value_t = 1)]
    height: usize,

    /// Size of the initially filled box
    #[arg(long, default_value_t = 500)]
    boxx: usize,
}

fn main() {
    let mpi_version = mpi::environment::library_version();
    let mpi_universe = if mpi_version.is_ok() {
        mpi::initialize_with_threading(mpi::Threading::Serialized)
    } else {
        None
    };

    let size = mpi_universe
        .as_ref()
        .map_or(1, |o: &(mpi::environment::Universe, mpi::Threading)| {
            o.0.world().size()
        });
    let rank = mpi_universe.as_ref().map_or(0, |o| o.0.world().rank());
    let previous_rank: Option<i32> = if rank == 0 { None } else { Some(rank - 1) };
    let next_rank: Option<i32> = if rank == (size - 1) as i32 {
        None
    } else {
        Some(rank + 1)
    };

    let cli = Cli::parse();

    let rounds = cli.rounds;
    let noise = cli.noise;
    let threads = cli.threads;
    let image_scaling = cli.scaling;
    let rounds_per_second = cli.speed;
    let frames_per_second = cli.framerate;
    let time_per_round = Duration::from_secs_f64(1.0 / rounds_per_second as f64);
    let time_per_frame = Duration::from_secs_f64(1.0 / (frames_per_second as f64).max(1.0));
    let height = (cli.height.div_ceil(size as usize).div_ceil(2)) * 2 as usize;
    let filepath = cli.output_directory.join(format!("output_{}.webp", rank));
    let filename = filepath.to_str().unwrap();

    if !cli.output_directory.is_dir() {
        if cli.output_directory.exists() {
            panic!("Output directory is not a directory");
        }
        std::fs::create_dir_all(&cli.output_directory).unwrap();
    }

    const WIDTH: usize = if cfg!(width_100000) {
        100000
    } else if cfg!(width_10000) {
        10000
    } else if cfg!(width_1000) {
        1000
    } else {
        100
    };
    // Put the correct number of threads into rayons global thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    // Create a sample section
    let mut sections_box = vec![[Cell::new(); WIDTH]; height];
    let mut sections_b_box = vec![[Cell::new(); WIDTH]; height];
    let mut grid_a: &mut [[Cell; WIDTH]] = sections_box.as_mut();
    let mut grid_b: &mut [[Cell; WIDTH]] = sections_b_box.as_mut();

    let box_y = cli
        .boxx
        .saturating_sub(height * (rank as usize))
        .min(height);
    let box_x = cli.boxx.min(WIDTH);

    grid_a[1][1].raw = 0b00111111;
    if previous_rank.is_none() {
        for x in 0..box_x {
            for y in 0..box_y {
                grid_a[y][x].raw = 0b00111111;
            }
        }
    }

    let random = &mut rand::thread_rng();
    for y in 0..height {
        for x in 0..WIDTH {
            if random.gen_bool(noise) {
                grid_a[y][x].raw ^= TO_EAST;
            }
            if random.gen_bool(noise) {
                grid_a[y][x].raw ^= TO_NORTH_EAST;
            }
            if random.gen_bool(noise) {
                grid_a[y][x].raw ^= TO_NORTH_WEST;
            }
            if random.gen_bool(noise) {
                grid_a[y][x].raw ^= TO_SOUTH_WEST;
            }
            if random.gen_bool(noise) {
                grid_a[y][x].raw ^= TO_SOUTH_EAST;
            }
            if random.gen_bool(noise) {
                grid_a[y][x].raw ^= TO_WEST;
            }
        }
    }
    eprintln!("============================ Round 0");
    let mut images: Vec<Image<Rgb>> = Vec::new();
    if frames_per_second != 0 {
        images.push(draw_cells_detailed(grid_a).resized(
            (WIDTH as f64 * image_scaling) as u32,
            (height as f64 * image_scaling) as u32,
            ril::ResizeAlgorithm::Lanczos3,
        ));
    }
    let mut top_bottom_duration: Duration = Duration::new(0, 0);
    let mut core_duration: Duration = Duration::new(0, 0);
    let mut communication_duration: Duration = Duration::new(0, 0);
    let mut render_duration: Duration = Duration::new(0, 0);

    let mut receive_top_box = Box::new([Cell::new(); WIDTH]);
    let mut receive_bottom_box = Box::new([Cell::new(); WIDTH]);
    let receive_top = receive_top_box.as_mut();
    let receive_bottom = receive_bottom_box.as_mut();

    let mut gif_time = Duration::new(0, 0);
    for round in 0..rounds {
        // process_round(grid_a, grid_b);
        let communication_time = Instant::now();
        if mpi_universe.is_some() {
            let world = SimpleCommunicator::world();
            mpi::request::scope(|scope| {
                let mut guards = Vec::new();

                if let Some(previous_rank) = previous_rank {
                    guards.push(WaitGuard::from(
                        world
                            .process_at_rank(previous_rank)
                            .immediate_send(scope, unsafe {
                                std::mem::transmute::<&mut [Cell; WIDTH], &mut [u8; WIDTH]>(
                                    &mut grid_a[0],
                                )
                            }),
                    ));
                    guards.push(WaitGuard::from(
                        world.process_at_rank(previous_rank).immediate_receive_into(
                            scope,
                            unsafe {
                                std::mem::transmute::<&mut [Cell; WIDTH], &mut [u8; WIDTH]>(
                                    receive_top,
                                )
                            },
                        ),
                    ));
                }

                if let Some(next_rank) = next_rank {
                    guards.push(WaitGuard::from(
                        world
                            .process_at_rank(next_rank)
                            .immediate_send(scope, unsafe {
                                std::mem::transmute::<&mut [Cell; WIDTH], &mut [u8; WIDTH]>(
                                    &mut grid_a[height - 1],
                                )
                            }),
                    ));
                    guards.push(WaitGuard::from(
                        world
                            .process_at_rank(next_rank)
                            .immediate_receive_into(scope, unsafe {
                                std::mem::transmute::<&mut [Cell; WIDTH], &mut [u8; WIDTH]>(
                                    receive_bottom,
                                )
                            }),
                    ));
                }
            });
        }
        communication_duration += communication_time.elapsed();

        let round_timer = Instant::now();
        if previous_rank.is_some() {
            movement_even_row(receive_top, &grid_a[0], &grid_a[1], &mut grid_b[0]);
        } else {
            movement_top_row(&grid_a[0], &grid_a[1], &mut grid_b[0]);
        }
        top_bottom_duration += round_timer.elapsed();

        let round_timer = Instant::now();
        grid_a
            .par_windows(3)
            .zip(grid_b.par_iter_mut().skip(1))
            .enumerate()
            .for_each(|(row_index, (context, result))| {
                let above = &context[0];
                let current = &context[1];
                let below = &context[2];
                if ((row_index + 1) % 2) == 0 {
                    movement_even_row(above, current, below, result);
                } else {
                    movement_odd_row(above, current, below, result);
                }
            });

        core_duration += round_timer.elapsed();

        let round_timer = Instant::now();
        if next_rank.is_some() {
            movement_odd_row(
                &grid_a[height - 2],
                &grid_a[height - 1],
                receive_bottom,
                &mut grid_b[height - 1],
            );
        } else {
            movement_bottom_row(
                &grid_a[height - 2],
                &grid_a[height - 1],
                &mut grid_b[height - 1],
            );
        }
        std::mem::swap(&mut grid_a, &mut grid_b);
        top_bottom_duration += round_timer.elapsed();

        if frames_per_second == 0 {
            continue;
        }
        let round_timer = Instant::now();
        gif_time += time_per_round;
        while gif_time >= time_per_frame {
            gif_time -= time_per_frame;
            eprintln!("============================ Round {}", round);
            images.push(draw_cells_detailed(grid_a).resized(
                (WIDTH as f64 * image_scaling) as u32,
                (height as f64 * image_scaling) as u32,
                ril::ResizeAlgorithm::Lanczos3,
            ));
        }
        render_duration += round_timer.elapsed();
    }

    let calculation_duration = core_duration + top_bottom_duration;

    let calculation_duration_per_cell =
        (calculation_duration.as_secs_f64() * 1000000000.0) / (WIDTH * height * rounds) as f64;
    let top_bottom_duration_per_cell =
        (top_bottom_duration.as_secs_f64() * 1000000000.0) / (WIDTH * 2 * rounds) as f64;
    let core_duration_per_cell =
        (core_duration.as_secs_f64() * 1000000000.0) / (WIDTH * (height - 2) * rounds) as f64;

    eprintln!(
        "Calculation duration per round: {}",
        calculation_duration.as_secs_f64() / rounds as f64
    );
    eprintln!(
        "Top/bottom duration per cell: {} ns",
        top_bottom_duration_per_cell
    );
    eprintln!("Core duration per cell: {} ns", core_duration_per_cell);
    eprintln!(
        "Calculation duration per cell: {} ns",
        calculation_duration_per_cell
    );
    eprintln!(
        "Calculation duration: {}",
        calculation_duration.as_secs_f64()
    );
    eprintln!(
        "Communication duration: {}",
        communication_duration.as_secs_f64()
    );
    eprintln!("Render duration: {}", render_duration.as_secs_f64());
    eprintln!(
        "Total duration: {}",
        (calculation_duration + communication_duration + render_duration).as_secs_f64()
    );

    println!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        WIDTH,
        height,
        rounds,
        size,
        threads,
        core_duration.as_secs_f64(),
        core_duration_per_cell,
        top_bottom_duration.as_secs_f64(),
        top_bottom_duration_per_cell,
        calculation_duration.as_secs_f64(),
        calculation_duration_per_cell,
        communication_duration.as_secs_f64(),
        (calculation_duration + communication_duration).as_secs_f64(),
        render_duration.as_secs_f64(),
        images.len()
    );

    if frames_per_second != 0 {
        let mut output = ImageSequence::<Rgb>::new();

        // ImageSequence::open is lazy
        for frame in images {
            let mut frame = Frame::from_image(frame);
            frame.set_delay(time_per_frame);
            output.push_frame(frame);
        }

        eprintln!("Saving output to {}", filename);

        let options = WebPEncoderOptions::new().with_lossless(true);
        let f = File::create(&filename).expect("Create file");
        let mut encoder =
            WebPMuxEncoder::new(f, EncoderMetadata::from(&output).with_config(options))
                .expect("new encoder");
        for frame in output.iter() {
            encoder.add_frame(frame).expect("adding frame");
        }
        encoder.finish().expect("finish");
        // output.save_inferred(filename).unwrap();
    }
    let _ = mpi_universe.as_ref().map_or(0, |o| o.0.world().rank());
}
