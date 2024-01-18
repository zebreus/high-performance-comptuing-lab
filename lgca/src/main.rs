#![feature(generic_const_exprs)]
#![feature(array_chunks)]
#![feature(iter_map_windows)]
#![feature(array_windows)]
#![feature(new_uninit)]
#![feature(split_array)]
#![feature(inline_const_pat)]

mod lgca;

use clap::Parser;
use mpi::traits::*;
use std::{path::PathBuf, time::Instant};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File containing the input data
    input: PathBuf,

    /// Where to write the output files. Must be a directory
    output_directory: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let input = cli.input;
    let output_directory = cli.output_directory;

    let mpi_version = mpi::environment::library_version();
    let mpi_universe = if mpi_version.is_ok() {
        mpi::initialize_with_threading(mpi::Threading::Single)
    } else {
        None
    };

    let rank = mpi_universe.as_ref().map_or(0, |o| o.0.world().rank());

    let input_file_exists = input.is_file();
    if rank == 0 && !input_file_exists {
        eprintln!("Input file {:?} does not exist or is not a file", input);
        std::process::exit(1);
    }

    let output_is_directory = output_directory.is_dir();
    if rank == 0 && !output_is_directory {
        eprintln!("Creating output directory {:?}", output_directory);
        std::fs::create_dir_all(&output_directory).unwrap();
    }

    let before_processing = Instant::now();

    // Content

    let duration = before_processing.elapsed();

    if rank == 0 {
        println!("{}", duration.as_secs_f64());
    }
}
