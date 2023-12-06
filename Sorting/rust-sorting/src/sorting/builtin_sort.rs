use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use crate::entry::Entry;

pub fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    let before_start = Instant::now();

    let read_bytes = std::fs::read(input_file).unwrap();
    // Read the input file into a vector of entries
    let mut input = read_bytes
        .array_chunks()
        .map(|chunk: &[u8; 100]| {
            let entry: &Entry = chunk.into();
            entry
        })
        .collect::<Vec<_>>();

    let read_duration = before_start.elapsed();
    eprintln!(
        "Finished reading in {} seconds.",
        read_duration.as_secs_f64()
    );

    let before_sort = Instant::now();

    // Perform a stable sort on the input
    input.sort_unstable();

    let sort_duration = before_sort.elapsed();
    eprintln!(
        "Finished sorting in {} seconds.",
        sort_duration.as_secs_f64()
    );

    let before_write = Instant::now();

    // Write the result to the output file
    let output_file_path = output_directory.join("output.sorted");
    let output_file = std::fs::File::create(&output_file_path).unwrap();
    let mut writer = BufWriter::new(output_file);
    input.into_iter().for_each(|entry| {
        let chunk: &[u8; 100] = entry.into();
        writer.write(chunk).unwrap();

        // return chunk.into_iter();
    }); // .collect::<Vec<u8>>();

    // output_file.write_all(&data).unwrap();
    writer.flush().unwrap();

    let write_duration = before_write.elapsed();
    eprintln!(
        "Finished writing in {} seconds.",
        write_duration.as_secs_f64()
    );

    return vec![output_file_path];
}
