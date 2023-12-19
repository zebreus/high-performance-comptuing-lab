#![feature(array_chunks)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
pub mod entry;
mod sorting;

use std::{path::PathBuf, process::exit, time::Instant};

use clap::Parser;
use mpi::traits::*;
use nix::NixPath;
use sorting::SortImplementation;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File containing the input data
    input: PathBuf,

    /// Where to write the output files. Must be a directory
    output_directory: PathBuf,

    /// Copy the input into the work_directory and work from there. Must be a directory
    #[arg(long)]
    work_directory: Option<PathBuf>,

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

    let mpi_version = mpi::environment::library_version();
    let mpi_universe = if mpi_version.is_ok() {
        // mpi::initialize();
        mpi::initialize_with_threading(mpi::Threading::Single)
    } else {
        None
    };

    let rank = mpi_universe.map_or(0, |o| o.0.world().rank());

    let input_file_exists = cli.input.is_file();
    if rank == 0 && !input_file_exists {
        eprintln!("Input file {:?} does not exist or is not a file", cli.input);
        std::process::exit(1);
    }
    let input_filename = cli.input.file_name().unwrap();
    let work_input: PathBuf = if rank == 0 {
        cli.work_directory
            .as_ref()
            .map_or(cli.input.clone(), |work_dir| {
                let work_input = work_dir.join(input_filename);
                if !work_input.is_file() {
                    eprintln!("Copying input file to work directory");
                    std::fs::copy(cli.input.as_path(), work_input.as_path()).unwrap();
                }
                work_input
            })
    } else {
        PathBuf::new()
    };

    if cli.output_directory.len() == 0 {
        eprintln!("Output directory is length zero");
        exit(1);
    }
    if cli.output_directory.is_file() {
        eprintln!("Output directory is a file");
        exit(1);
    }

    let output_is_directory = cli.output_directory.is_dir();
    if !output_is_directory {
        eprintln!("Creating output directory {:?}", cli.output_directory);
        std::fs::create_dir_all(&cli.output_directory).unwrap();
    }

    // let setup_duration = before_setup.elapsed();
    // eprintln!("setup time = {} seconds", setup_duration.as_secs_f64());
    let before_sort = Instant::now();

    algorithm
        .sort(work_input.as_path(), cli.output_directory.as_path())
        .await;

    let sort_duration = before_sort.elapsed();

    // if proc.is_some() {}
    if rank == 0 {
        println!("{}", sort_duration.as_secs_f64());
    }
    // eprintln!("Output files: {:?}", output_files);
}
