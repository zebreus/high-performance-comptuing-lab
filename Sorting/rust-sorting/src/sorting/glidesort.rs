use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use rdst::RadixSort;

use crate::entry::{entries_to_u8_unsafe, u8_to_entries_unsafe};

pub fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    let before_start = Instant::now();

    let read_bytes = std::fs::read(input_file).unwrap();
    let mut input = u8_to_entries_unsafe(read_bytes);

    let read_duration = before_start.elapsed();
    eprintln!(
        "Finished reading in {} seconds.",
        read_duration.as_secs_f64()
    );

    let before_sort = Instant::now();

    input
        .radix_sort_builder()
        .with_single_threaded_tuner()
        .with_parallel(false)
        .sort();

    let temp = &input[0..10];
    let debug_buffers = &temp
        .into_iter()
        .map(|entry| entry.key())
        .collect::<Vec<_>>();
    // Check if we are done
    println!("BUFFERS: {:?}", debug_buffers);

    let sort_duration = before_sort.elapsed();
    eprintln!(
        "Finished sorting in {} seconds.",
        sort_duration.as_secs_f64()
    );

    let before_write = Instant::now();

    let data = entries_to_u8_unsafe(input);
    let output_file_path = output_directory.join("output.sorted");
    // let mut output_file = std::fs::File::create(&output_file_path).unwrap();
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .truncate(true)
        .open(&output_file_path)
        .unwrap();
    // let mut writer = BufWriter::new(output_file);
    output_file.write_all(&data).unwrap();
    // output_file.sync_all().unwrap();

    // writer.flush().unwrap();
    // writer.into_inner().unwrap().sync_all().unwrap();
    // output_file.sync_all().unwrap();

    let write_duration = before_write.elapsed();
    eprintln!(
        "Finished writing in {} seconds.",
        write_duration.as_secs_f64()
    );

    eprintln!("in glidesort impl");

    return vec![output_file_path];
}
