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

    /// Print the result matrix instead of the time to stdout
    #[arg(short, long)]
    print_matrix: Option<bool>,
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
    let matrix_b = Matrix::from_file(cli.matrix_a.as_str()).unwrap();
    let setup_duration = before_setup.elapsed();
    let print_matrix = cli.print_matrix.unwrap_or(false);
    eprintln!("setup time = {} seconds", setup_duration.as_secs_f64());

    let before_calculation = Instant::now();
    let result = matrix_a.multiply(&matrix_b).unwrap();
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
