#![feature(array_chunks)]
#![feature(noop_waker)]
pub mod entry;
mod sorting;

use std::{path::PathBuf, time::Instant};

use clap::Parser;
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let before_setup = Instant::now();

    // Put the correct number of threads into rayons global thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads.unwrap_or(1))
        .build_global()
        .unwrap();

    let algorithm = cli.algorithm;
    let output_is_directory = cli.output_directory.is_dir();
    if !output_is_directory {
        eprintln!(
            "Output directory {:?} does not exist or is not a directory",
            cli.output_directory
        );
        std::process::exit(1);
    }

    let setup_duration = before_setup.elapsed();
    eprintln!("setup time = {} seconds", setup_duration.as_secs_f64());
    let before_sort = Instant::now();

    let output_files = algorithm
        .sort(cli.input.as_path(), cli.output_directory.as_path())
        .await;

    eprintln!("in main");
    let sort_duration = before_sort.elapsed();

    println!("{}", sort_duration.as_secs_f64());
    eprintln!("Output files: {:?}", output_files);
}
