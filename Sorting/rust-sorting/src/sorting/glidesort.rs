use std::{
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
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

    eprintln!("starting sort");
    let before_sort = Instant::now();

    // Perform a stable sort on the input
    glidesort::sort(&mut input);

    let sort_duration = before_sort.elapsed();
    eprintln!("sort time = {} seconds", sort_duration.as_secs_f64());

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
