pub mod matrix;
use std::time::Instant;

use clap::{Parser, ValueEnum};
use matrix::Matrix;

#[derive(ValueEnum, Debug, PartialEq, Clone)]
#[clap(rename_all = "kebab_case")]
enum MultiplyImplementation {
    NoIndices,
    FaithfulPairs,
    FaithfulMutex,
    FaithfulUnsafe,
    FaithfulIterators,
}
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

    /// Print the result matrix instead of the time to stdout
    #[arg(short, long)]
    print_matrix: Option<bool>,

    /// File containing the second matrix
    #[arg(short, long, default_value = "no_indices")]
    algorithm: MultiplyImplementation,
}

impl MultiplyImplementation {
    fn multiply(&self, a: &Matrix, b: &Matrix) -> Matrix {
        match self {
            MultiplyImplementation::NoIndices => a.multiply(b),
            MultiplyImplementation::FaithfulPairs => a.multiply_faithful_pairs(b),
            MultiplyImplementation::FaithfulMutex => a.multiply_faithful_mutex(b),
            MultiplyImplementation::FaithfulUnsafe => a.multiply_faithful_unsafe(b),
            MultiplyImplementation::FaithfulIterators => a.multiply_faithful_iterators(b),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let before_setup = Instant::now();

    // Put the correct number of threads into rayons global thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads.unwrap_or(1))
        .build_global()
        .unwrap();

    let matrix_a = Matrix::from_file(cli.matrix_a.as_str()).unwrap();
    let matrix_b = Matrix::from_file(cli.matrix_b.as_str()).unwrap();
    let setup_duration = before_setup.elapsed();
    let print_matrix = cli.print_matrix.unwrap_or(false);
    let algorithm = cli.algorithm;
    eprintln!("setup time = {} seconds", setup_duration.as_secs_f64());

    assert_eq!(matrix_a.cols, matrix_b.rows);

    let before_calculation = Instant::now();
    let result = algorithm.multiply(&matrix_a, &matrix_b);
    let calculation_duration = before_calculation.elapsed();
    if print_matrix {
        print!("{}", result);
    } else {
        println!("{},{}", calculation_duration.as_secs_f64(), result.sum());
    }
    eprintln!(
        "calculation time = {} seconds",
        calculation_duration.as_secs_f64()
    );

    eprintln!("sum of the result = {}", result.sum());
}
