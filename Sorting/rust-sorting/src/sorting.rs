mod builtin_sort;
mod glidesort;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

#[derive(ValueEnum, Debug, PartialEq, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum SortImplementation {
    Glidesort,
    BuiltinSort,
}

impl SortImplementation {
    pub async fn sort(&self, input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
        match self {
            SortImplementation::Glidesort => glidesort::sort(input_file, output_directory).await,
            SortImplementation::BuiltinSort => {
                builtin_sort::sort(input_file, output_directory).await
            }
        }
    }
}
