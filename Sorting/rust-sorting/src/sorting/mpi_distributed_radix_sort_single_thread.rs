use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use mpi::{request::WaitGuard, Rank};
use mpi::{topology::SimpleCommunicator, traits::*};
use nix::libc::{posix_fadvise64, POSIX_FADV_NOREUSE, POSIX_FADV_SEQUENTIAL, POSIX_FADV_WILLNEED};

use crate::entry::{boxed_u8_to_entries, entries_to_u8_unsafe, Entry, RadixDivider, SortedEntries};

use super::{
    mpi_distributed_radix_sort::{MIN_READ_BLOCK_SIZE, READ_BLOCK_SIZE},
    BLOCK_SIZE,
};

pub fn get_worker(bucket_id: i32, workers: i32) -> Rank {
    return ((bucket_id as i32) % workers) + 1;
}

pub fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    // let universe = mpi::initialize().unwrap();
    let world = SimpleCommunicator::world();
    let size = world.size();
    let rank = world.rank();

    let mut time_spend_reading_the_input = Duration::new(0, 0);
    let mut time_spend_dividing_the_input_into_buckets = Duration::new(0, 0);
    let mut time_spend_sending_to_workers = Duration::new(0, 0);
    let mut time_spend_receiving_on_worker = Duration::new(0, 0);
    let mut time_spend_sorting_on_worker = Duration::new(0, 0);
    let mut time_spend_writing_to_disk = Duration::new(0, 0);
    let mut time_spend_receiving_from_workers = Duration::new(0, 0);
    let mut time_spend_sending_to_manager = Duration::new(0, 0);
    let mut time_spend_fetching_time_from_workers = Duration::new(0, 0);

    if rank == 0 {
        let file = std::fs::File::open(input_file).unwrap();
        let file_length = file.metadata().unwrap().len();
        unsafe {
            let fd = file.as_raw_fd();
            let fadvise_result = posix_fadvise64(fd, 0, file_length as i64, POSIX_FADV_SEQUENTIAL);
            assert_eq!(fadvise_result, 0);
            let fadvise_result = posix_fadvise64(fd, 0, file_length as i64, POSIX_FADV_NOREUSE);
            assert_eq!(fadvise_result, 0);
        };

        let mut radix_divider = RadixDivider::new();
        let mut reader = file;
        let mut read_buffer: Box<[u8; READ_BLOCK_SIZE]> = Box::new([0; READ_BLOCK_SIZE]);
        loop {
            let before_read = Instant::now();
            let mut pos = 0usize;
            {
                let mut current_end = 0;

                while current_end % 100 != 0
                    || current_end == 0
                    || current_end < MIN_READ_BLOCK_SIZE
                {
                    let length = reader.read(&mut read_buffer[current_end..]).unwrap();
                    // eprintln!("Read {} bytes", length);
                    pos += length;
                    if length == 0 {
                        break;
                    }
                    current_end += length;
                }
                // Inform about the next read
                unsafe {
                    let fd = reader.as_raw_fd();
                    let read_length = (READ_BLOCK_SIZE % (4096 * 512)) * (4096 * 512);
                    let fadvise_result =
                        posix_fadvise64(fd, pos as i64, read_length as i64, POSIX_FADV_WILLNEED);
                    assert_eq!(fadvise_result, 0);
                };
            }
            if pos == 0 {
                break;
            }
            time_spend_reading_the_input += before_read.elapsed();
            let before_processing = Instant::now();

            let my_buffer = &read_buffer[..pos];
            let entries = unsafe {
                assert!(pos % 100 == 0);
                core::slice::from_raw_parts(my_buffer.as_ptr() as *const Entry, pos / 100)
            };

            radix_divider.push_all(entries);
            time_spend_dividing_the_input_into_buckets += before_processing.elapsed();

            let before_send = Instant::now();
            if radix_divider.ready_to_delegate() {
                let full_buffers = radix_divider.borrow_delegateable_buffers();

                mpi::request::scope(|scope| {
                    let _ = full_buffers
                        .into_iter()
                        .map(|(root, block)| {
                            if block.len() == 0 {
                                return None;
                            }
                            let target = get_worker(root as i32, size - 1);
                            Some(WaitGuard::from(
                                world.process_at_rank(target).immediate_send(scope, block),
                            ))
                        })
                        .collect::<Vec<_>>();
                });
            }
            time_spend_sending_to_workers += before_send.elapsed();
        }

        let before_send = Instant::now();
        // Send remaining buffers
        let full_buffers = radix_divider.borrow_delegateable_buffers();
        mpi::request::scope(|scope| {
            let _ = full_buffers
                .into_iter()
                .map(|(root, block)| {
                    if block.len() == 0 {
                        return None;
                    }
                    let target = get_worker(root as i32, size - 1);
                    Some(WaitGuard::from(
                        world.process_at_rank(target).immediate_send(scope, block),
                    ))
                })
                .collect::<Vec<_>>();
        });

        // Send done to all workers
        for i in 0..(size - 1) {
            world.process_at_rank(i + 1).send(&[42u8]);
        }
        time_spend_sending_to_workers += before_send.elapsed();

        let output_file_path = output_directory.join("output.sorted");
        let mut output_file_writer = std::fs::File::create(&output_file_path).unwrap();
        // let mut output_file_writer = BufWriter::new(output_file);
        // A bit more than 1 Bucket
        // If the program crashes, it was probably because of this line
        let mut receive_buffer = vec![0u8; ((file_length as usize / 256) * 3 / 2) + 5000];
        for bucket_id in 0..=255 {
            let receive_start = Instant::now();
            let node: i32 = get_worker(bucket_id, size - 1);
            let status = world
                .process_at_rank(node)
                .receive_into(&mut receive_buffer);
            let length = status.count(0u8.as_datatype()) as usize;
            let received_data = &receive_buffer[..length];
            time_spend_receiving_from_workers += receive_start.elapsed();

            let before_write = Instant::now();
            output_file_writer.write_all(received_data).unwrap();
            time_spend_writing_to_disk += before_write.elapsed();
        }
        // Make sure our measurements include actually writing to disk and not just cache
        let before_flush = Instant::now();
        output_file_writer.flush().unwrap();
        output_file_writer.sync_data().unwrap();
        time_spend_writing_to_disk += before_flush.elapsed();
    }

    if rank != 0 {
        let mut buffers: [Option<SortedEntries>; 256] = arr_macro::arr![None; 256];

        // Solution one: Completly synchronous
        loop {
            let before_receiving = Instant::now();
            let mut data: Box<[u8; BLOCK_SIZE * 100 * 2]> = Box::new([0; BLOCK_SIZE * 100 * 2]);
            let status = world.process_at_rank(0).receive_into(&mut *data);
            let length = status.count(0u8.as_datatype()) as usize;
            if length == 1 && data[0] == 42 {
                // We received the signal to stop
                break;
            }
            let received_entries = boxed_u8_to_entries(data, length);
            time_spend_receiving_on_worker += before_receiving.elapsed();

            let before_sorting = Instant::now();
            let bucket = received_entries[0].bucket();
            match buffers[bucket] {
                Some(ref mut buffer) => {
                    buffer.join(received_entries);
                }
                None => {
                    buffers[bucket] = Some(received_entries.into());
                }
            }
            time_spend_sorting_on_worker += before_sorting.elapsed();
        }

        let send_start = Instant::now();
        buffers
            .into_iter()
            .enumerate()
            .for_each(|(bucket, buffer)| {
                if rank != get_worker(bucket as i32, size - 1) {
                    return;
                }
                let buffer = buffer.map(|b| b.into_vec()).unwrap_or(Vec::new());
                let result_vec = entries_to_u8_unsafe(buffer);
                world.process_at_rank(0).send(result_vec.as_slice());
            });
        time_spend_sending_to_manager += send_start.elapsed();
    }

    // Get the mean of the benchmark times from the workers
    let fetch_time_start = Instant::now();
    if rank == 0 {
        let mut all_worker_times: [Vec<Duration>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for i in 1..size {
            let (worker_times, _) = world.process_at_rank(i).receive_vec::<u64>();
            all_worker_times[0].push(Duration::from_micros(worker_times[0]));
            all_worker_times[1].push(Duration::from_micros(worker_times[1]));
            all_worker_times[2].push(Duration::from_micros(worker_times[2]));
        }
        all_worker_times[0].sort_unstable();
        all_worker_times[1].sort_unstable();
        all_worker_times[2].sort_unstable();
        time_spend_receiving_on_worker = all_worker_times[0][(size as usize - 1) / 2];
        time_spend_sorting_on_worker = all_worker_times[1][(size as usize - 1) / 2];
        time_spend_sending_to_manager = all_worker_times[2][(size as usize - 1) / 2];
    }
    if rank != 0 {
        world.process_at_rank(0).send(&[
            time_spend_receiving_on_worker.as_micros() as u64,
            time_spend_sorting_on_worker.as_micros() as u64,
            time_spend_sending_to_manager.as_micros() as u64,
        ]);
    }
    time_spend_fetching_time_from_workers += fetch_time_start.elapsed();

    if rank == 0 {
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
    }

    return vec![];
}
