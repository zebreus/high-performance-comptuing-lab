use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::entry::Entry;

pub async fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    // Read the input file into a vector of entries
    let mut input = std::fs::read(input_file)
        .unwrap()
        .array_chunks()
        .map(|chunk: &[u8; 100]| {
            let entry: Entry = chunk.into();
            entry
        })
        .collect::<Vec<_>>();

    // Perform a stable sort on the input
    input.sort();

    // Write the result to the output file
    let output_file_path = output_directory.join("output.sorted");
    let mut output_file = std::fs::File::create(&output_file_path).unwrap();
    input.into_iter().for_each(|entry| {
        let chunk: [u8; 100] = entry.into();
        output_file.write(&chunk).unwrap();
    });
    output_file.sync_all().unwrap();

    return vec![output_file_path];
}
