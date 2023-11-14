pub mod matrix;

use std::time::Instant;

use clap::Parser;
use matrix::Matrix;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File containing the first matrix
    matrix_a: String,

    /// File containing the second matrix
    matrix_b: String,

    /// Set the number of threads to use
    #[arg(short, long)]
    threads: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    // Put the correct number of threads into rayons global thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads.unwrap_or(1))
        .build_global()
        .unwrap();

    let matrix_a = Matrix::from_file(cli.matrix_a.as_str()).unwrap();
    let matrix_b = Matrix::from_file(cli.matrix_a.as_str()).unwrap();

    let start = Instant::now();
    let result = matrix_a.multiply(&matrix_b).unwrap();
    let duration = start.elapsed();

    println!("{:?}", duration.as_secs_f64());
    eprintln!("Sum of all values: {:?}", result.sum());
}
