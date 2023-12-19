use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use rdst::RadixSort;

use crate::entry::{entries_to_u8_unsafe, u8_to_entries_unsafe};

pub fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    let mut time_spend_reading_the_input = Duration::new(0, 0);
    let time_spend_dividing_the_input_into_buckets = Duration::new(0, 0);
    let time_spend_sending_to_workers = Duration::new(0, 0);
    let time_spend_receiving_on_worker = Duration::new(0, 0);
    let mut time_spend_sorting_on_worker = Duration::new(0, 0);
    let mut time_spend_writing_to_disk = Duration::new(0, 0);
    let time_spend_receiving_from_workers = Duration::new(0, 0);
    let time_spend_sending_to_manager = Duration::new(0, 0);
    let time_spend_fetching_time_from_workers = Duration::new(0, 0);

    let before_start = Instant::now();

    let read_bytes = std::fs::read(input_file).unwrap();
    let mut input = u8_to_entries_unsafe(read_bytes);

    time_spend_reading_the_input += before_start.elapsed();

    let before_sort = Instant::now();

    input
        .radix_sort_builder()
        .with_single_threaded_tuner()
        .with_parallel(false)
        .sort();

    time_spend_sorting_on_worker += before_sort.elapsed();

    let before_write = Instant::now();

    let data = entries_to_u8_unsafe(input);

    let output_file_path = output_directory.join("output.sorted");
    let mut output_file = std::fs::File::create(&output_file_path).unwrap();
    output_file.write_all(&data).unwrap();

    time_spend_writing_to_disk += before_write.elapsed();

    print!(
        "{},{},{},{},{},{},{},{},{},",
        time_spend_reading_the_input.as_secs_f64(),
        time_spend_dividing_the_input_into_buckets.as_secs_f64(),
        time_spend_sending_to_workers.as_secs_f64(),
        time_spend_writing_to_disk.as_secs_f64(),
        time_spend_receiving_from_workers.as_secs_f64(),
        time_spend_fetching_time_from_workers.as_secs_f64(),
        time_spend_receiving_on_worker.as_secs_f64(),
        time_spend_sorting_on_worker.as_secs_f64(),
        time_spend_sending_to_manager.as_secs_f64(),
    );
    let total_time_distributing = time_spend_reading_the_input
        + time_spend_dividing_the_input_into_buckets
        + time_spend_sending_to_workers;
    let total_time_collecting = time_spend_receiving_from_workers + time_spend_writing_to_disk;
    eprintln!(
        "Distributing took {:.8} seconds.",
        (total_time_distributing).as_secs_f64()
    );
    eprintln!(
        "Collecting took {:.8} seconds.",
        (total_time_collecting).as_secs_f64()
    );
    eprintln!("");
    eprintln!(
        "Read took {:.8} seconds.",
        time_spend_reading_the_input.as_secs_f64()
    );
    eprintln!(
        "Processing took {:.8} seconds.",
        time_spend_dividing_the_input_into_buckets.as_secs_f64()
    );
    eprintln!(
        "Send took {:.8} seconds.",
        time_spend_sending_to_workers.as_secs_f64()
    );
    eprintln!(
        "Write took {:.8} seconds.",
        time_spend_writing_to_disk.as_secs_f64()
    );
    eprintln!(
        "Receive took {:.8} seconds.",
        time_spend_receiving_from_workers.as_secs_f64()
    );
    eprintln!(
        "Fetching time from workers took {:.8} seconds.",
        time_spend_fetching_time_from_workers.as_secs_f64()
    );
    eprintln!(
        "Receiving on worker took {:.8} seconds.",
        time_spend_receiving_on_worker.as_secs_f64()
    );
    eprintln!(
        "Actually sorting took {:.8} seconds.",
        time_spend_sorting_on_worker.as_secs_f64()
    );
    eprintln!(
        "Sending back to manager took {:.8} seconds.",
        time_spend_sending_to_manager.as_secs_f64()
    );

    return vec![output_file_path];
}
