#![feature(array_chunks)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
pub mod entry;
mod sorting;

use std::{path::PathBuf, time::Instant};

use clap::Parser;
use mpi::traits::*;
use sorting::SortImplementation;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File containing the input data
    input: PathBuf,

    /// Where to write the output files. Must be a directory
    output_directory: PathBuf,

    /// Set the number of threads to use
    #[arg(short, long)]
    threads: Option<usize>,

    /// Select the sorting algorithm to use
    #[arg(short, long, default_value = "no-indices")]
    algorithm: SortImplementation,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = Cli::parse();

    // Put the correct number of threads into rayons global thread pool
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(cli.threads.unwrap_or(1))
    //     .build_global()
    //     .unwrap();

    let algorithm = cli.algorithm;
    let output_is_directory = cli.output_directory.is_dir();
    if !output_is_directory {
        eprintln!(
            "Output directory {:?} does not exist or is not a directory",
            cli.output_directory
        );
        std::process::exit(1);
    }

    let mpi_version = mpi::environment::library_version();
    let mpi_universe = if mpi_version.is_ok() {
        // mpi::initialize();
        mpi::initialize_with_threading(mpi::Threading::Single)
    } else {
        None
    };

    // let setup_duration = before_setup.elapsed();
    // eprintln!("setup time = {} seconds", setup_duration.as_secs_f64());
    let before_sort = Instant::now();

    algorithm
        .sort(cli.input.as_path(), cli.output_directory.as_path())
        .await;

    let sort_duration = before_sort.elapsed();

    // if proc.is_some() {}
    let rank = mpi_universe.map_or(0, |o| o.0.world().rank());
    if rank == 0 {
        println!("{}", sort_duration.as_secs_f64());
    }
    // eprintln!("Output files: {:?}", output_files);
}
