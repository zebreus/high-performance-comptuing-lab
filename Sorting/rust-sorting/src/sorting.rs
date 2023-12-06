mod builtin_sort;
mod glidesort;
mod mpi_block_sort;
mod mpi_distributed_radix_sort;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

#[derive(ValueEnum, Debug, PartialEq, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum SortImplementation {
    Glidesort,
    BuiltinSort,
    MpiBlockSort,
    MpiDistributedRadixSort,
}

impl SortImplementation {
    pub async fn sort(&self, input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
        match self {
            SortImplementation::Glidesort => glidesort::sort(input_file, output_directory),
            SortImplementation::BuiltinSort => builtin_sort::sort(input_file, output_directory),
            SortImplementation::MpiBlockSort => mpi_block_sort::sort(input_file, output_directory),
            SortImplementation::MpiDistributedRadixSort => {
                mpi_distributed_radix_sort::sort(input_file, output_directory).await
            }
        }
    }
}
