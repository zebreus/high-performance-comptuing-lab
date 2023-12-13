use core::panic;
use std::{
    io::{BufWriter, Write},
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::future::join_all;
use mpi::{request::WaitGuard, Rank};
use mpi::{topology::SimpleCommunicator, traits::*};
use nix::libc::{posix_fadvise64, POSIX_FADV_NOREUSE, POSIX_FADV_SEQUENTIAL, POSIX_FADV_WILLNEED};
use tokio::{
    io::AsyncReadExt,
    runtime::{Handle, Runtime},
    spawn,
    sync::Mutex,
};

use crate::entry::{
    entries_to_u8_unsafe, u8_to_entries_unsafe, Entry, RadixDivider, SortedEntries,
};

/// Size of the blocks of entries that will be transmitted to one worker in one go. In entries/ 100bytes
const BLOCK_SIZE: usize = 10000; // 10000 Entries = 1 MB
/// Size of datablocks that will be read from disk. In bytes
const READ_BLOCK_SIZE: usize = 256 * 100 * 100;
/// Minimum size read before starting to sort. In bytes
/// Has to be smaller than READ_BLOCK_SIZE
const MIN_READ_BLOCK_SIZE: usize = 0;

// Manager uses 2*READ_BLOCK_SIZE + 256*4*100*BLOCK_SIZE bytes of memory
// The workers use the (TOTAL_DATA_SIZE/NUMBER_OF_WORKERS)*2 bytes of memory

macro_rules! measure_time {
    // This macro takes an expression of type `expr` and prints
    // it as a string along with its result.
    // The `expr` designator is used for expressions.
    ( $name:expr, $preparation:block   ) => {
        // `stringify!` will convert the expression *as it is* into a string.
        let before = Instant::now();
        $preparation
        let duration = before.elapsed();
        eprintln!(
            "{} took {} micros.",
            $name,
            duration.as_micros()
        );
    };
}

// macro_rules! mark {
//     // This macro takes an expression of type `expr` and prints
//     // it as a string along with its result.
//     // The `expr` designator is used for expressions.
//     ( $name:expr, $preparation:expr   ) => {
//         // `stringify!` will convert the expression *as it is* into a string.
//         {
//             let before = Instant::now();
//             let value = $preparation;
//             let duration = before.elapsed();
//             eprintln!("{} took {} micros.", $name, duration.as_micros());
//             value
//         }
//     };
// }

pub fn get_worker(bucket_id: i32, workers: i32) -> Rank {
    return ((bucket_id as i32) % workers) + 1;
}

pub async fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
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
        let file = tokio::fs::File::open(input_file).await.unwrap();
        let file_length = file.metadata().await.unwrap().len();
        measure_time!("distributing", {
            unsafe {
                let fd = file.as_raw_fd();
                let fadvise_result =
                    posix_fadvise64(fd, 0, file_length as i64, POSIX_FADV_SEQUENTIAL);
                assert_eq!(fadvise_result, 0);
                let fadvise_result = posix_fadvise64(fd, 0, file_length as i64, POSIX_FADV_NOREUSE);
                assert_eq!(fadvise_result, 0);
            };

            let mut radix_divider = RadixDivider::<BLOCK_SIZE>::new();

            let reader = Arc::new(Mutex::new((0, file)));
            let work_box: Arc<Mutex<Box<[u8; READ_BLOCK_SIZE]>>> =
                Arc::new(Mutex::new(Box::new([0; READ_BLOCK_SIZE])));
            let read_box: Arc<Mutex<Box<[u8; READ_BLOCK_SIZE]>>> =
                Arc::new(Mutex::new(Box::new([0; READ_BLOCK_SIZE])));
            let mut work_buffer_end: usize = 0;

            let mut send_task: Option<tokio::task::JoinHandle<()>> = Option::None;
            let mut send_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();

            let mut read_task: Option<tokio::task::JoinHandle<usize>> = Option::None;

            loop {
                let before_read = Instant::now();
                {
                    match read_task {
                        Some(task) => {
                            work_buffer_end = task.await.unwrap();
                            if work_buffer_end == 0 {
                                break;
                            }
                        }
                        None => {}
                    }
                }
                {
                    let reader_and_pos = reader.clone();
                    let read_box = read_box.clone();
                    let work_box = work_box.clone();

                    read_task = Some(tokio::spawn(async move {
                        let mut current_end = 0;
                        let mut reader_and_pos = reader_and_pos.lock().await;
                        unsafe {
                            let fd = reader_and_pos.1.as_raw_fd();
                            let read_length = (READ_BLOCK_SIZE % (4096 * 512)) * (4096 * 512);
                            let fadvise_result = posix_fadvise64(
                                fd,
                                reader_and_pos.0,
                                read_length as i64,
                                POSIX_FADV_WILLNEED,
                            );
                            assert_eq!(fadvise_result, 0);
                        };
                        let mut read_box = read_box.lock().await;
                        let read_buffer = &mut **read_box;
                        while current_end % 100 != 0
                            || current_end == 0
                            || current_end < MIN_READ_BLOCK_SIZE
                        {
                            let length = reader_and_pos
                                .1
                                .read_buf(&mut &mut read_buffer[current_end..])
                                .await
                                .unwrap();
                            reader_and_pos.0 += length as i64;
                            if length == 0 {
                                break;
                            }
                            current_end += length;
                        }

                        let mut target_box = work_box.lock().await;

                        std::mem::swap(&mut *target_box, &mut *read_box);
                        return current_end;
                    }));
                }
                time_spend_reading_the_input += before_read.elapsed();
                let before_processing = Instant::now();
                let my_work_box = work_box.lock().await;

                let my_buffer = &my_work_box[..work_buffer_end];
                let entries = unsafe {
                    assert!(work_buffer_end % 100 == 0);
                    core::slice::from_raw_parts(
                        my_buffer.as_ptr() as *const Entry,
                        work_buffer_end / 100,
                    )
                };

                radix_divider.push_all(entries);

                time_spend_dividing_the_input_into_buckets += before_processing.elapsed();
                let before_send = Instant::now();

                let all_buffers_ready = radix_divider.ready_to_delegate();
                if all_buffers_ready {
                    let full_buffers = radix_divider.get_delegateable_buffers();
                    if let Some(send_task) = send_task.take() {
                        send_task.await.unwrap();
                    }
                    send_task = Some(spawn(async move {
                        let world = SimpleCommunicator::world();
                        mpi::request::scope(|scope| {
                            let mut guards = Vec::new();
                            for buffer in &full_buffers {
                                let (root, block) = buffer;
                                // eprintln!(
                                //     "Sending block {} to worker {} with length {}",
                                //     root,
                                //     target,
                                //     block.len()
                                // );
                                if block.len() == 0 {
                                    return;
                                }
                                let target = get_worker(*root as i32, size - 1);

                                // eprintln!("Sending message to process {}", target + 1);
                                guards.push(WaitGuard::from(
                                    world.process_at_rank(target).immediate_send(scope, block),
                                ));
                            }
                            guards.clear();
                        });
                    }));
                    // handle.await.unwrap();
                }
                time_spend_sending_to_workers += before_send.elapsed();
            }
            if let Some(send_task) = send_task.take() {
                send_task.await.unwrap();
            }
            let before_send = Instant::now();
            let full_buffers = radix_divider.get_delegateable_buffers();
            mpi::request::scope(|scope| {
                let mut guards = Vec::new();
                for buffer in &full_buffers {
                    let (root, block) = buffer;
                    if block.len() == 0 {
                        return;
                    }
                    let target = get_worker(*root as i32, size - 1);

                    // eprintln!("Sending message to process {}", target + 1);
                    guards.push(WaitGuard::from(
                        world.process_at_rank(target).immediate_send(scope, block),
                    ));
                }
            });
            time_spend_sending_to_workers += before_send.elapsed();

            for i in 0..(size - 1) {
                world.process_at_rank(i + 1).send(&[42u8]);
            }
            time_spend_sending_to_workers += before_send.elapsed();
        });

        // Now we have distributed that stuff, lets try to get the sorted data back

        let mut writer_thread: Option<tokio::task::JoinHandle<()>> = Option::None;
        let output_file_path = output_directory.join("output.sorted");
        let output_file = std::fs::File::create(&output_file_path).unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(output_file)));

        for bucket_id in 0..=255 {
            // A bit more than 1 Bucket
            let mut receive_buffer = vec![0u8; (file_length as usize / 256) * 11 / 10];
            let receive_start = Instant::now();
            let node = get_worker(bucket_id, size - 1);
            let status = world
                .process_at_rank(node)
                .receive_into(&mut receive_buffer);
            let bytes = 500 as usize;
            // let data = ;
            if bytes == 1 && receive_buffer[0] == 42 {
                panic!("Got done from node {} when expecting a result", node);
            }
            time_spend_receiving_from_workers += receive_start.elapsed();
            let before_write = Instant::now();
            if let Some(thread) = writer_thread.take() {
                thread.await.unwrap();
            }
            let writer = Arc::clone(&writer);
            writer_thread = Some(spawn(async move {
                let mut writer = writer.lock().await;
                writer.write_all(&receive_buffer[..bytes]).unwrap();
            }));
            time_spend_writing_to_disk += before_write.elapsed();
        }

        let before_write = Instant::now();
        if let Some(thread) = writer_thread.take() {
            thread.await.unwrap();
        }
        writer.lock().await.flush().unwrap();
        time_spend_writing_to_disk += before_write.elapsed();
    }

    if rank != 0 {
        let mut buffers: [Option<SortedEntries>; 256] = arr_macro::arr![None; 256];

        // Solution one: Completly synchronous
        loop {
            let mut data = Vec::<u8>::with_capacity(BLOCK_SIZE * 100 * 2);
            let before_receiving = Instant::now();
            unsafe {
                data.set_len(BLOCK_SIZE * 100 * 2);
                let status = world.process_at_rank(0).receive_into(&mut *data);
                let length = status.count(0u8.as_datatype()) as usize;
                data.set_len(length);
            };
            time_spend_receiving_on_worker += before_receiving.elapsed();
            let before_sorting = Instant::now();
            if data.len() == 1 && data[0] == 42 {
                break;
            }
            let root = data[0].clone();
            let input = u8_to_entries_unsafe(data);
            if buffers[root as usize].is_some() {
                buffers[root as usize].as_mut().unwrap().join(input);
            } else {
                buffers[root as usize] = Some(input.into());
            }
            time_spend_sorting_on_worker += before_sorting.elapsed();
        }

        // // Solution two: Asynchronous receiving using openMPI. No benefit
        // let mut buffer_a = Box::new([0u8; 2 * BLOCK_SIZE * 100]);
        // let mut buffer_b = Box::new([0u8; 2 * BLOCK_SIZE * 100]);
        // let mut length: usize = 0;
        // loop {
        //     mpi::request::scope(|scope_b| {
        //         let before_receiving = Instant::now();
        //         let world = SimpleCommunicator::world();
        //         let request = world
        //             .process_at_rank(0)
        //             .immediate_receive_into(scope_b, &mut *buffer_a);
        //         time_spend_receiving_on_worker += before_receiving.elapsed();
        //
        //         let before_sorting = Instant::now();
        //         let root = buffer_b[0].clone();
        //         let input = unsafe {
        //             assert!(length % 100 == 0);
        //             core::slice::from_raw_parts(buffer_b.as_ptr() as *const Entry, length / 100)
        //         };
        //         if buffers[root as usize].is_some() {
        //             buffers[root as usize]
        //                 .as_mut()
        //                 .unwrap()
        //                 .join(input.to_vec());
        //         } else {
        //             buffers[root as usize] = Some(input.to_vec().into());
        //         }
        //         time_spend_sorting_on_worker += before_sorting.elapsed();
        //
        //         let before_receiving = Instant::now();
        //         let status = request.wait();
        //         length = status.count(0u8.as_datatype()) as usize;
        //         time_spend_receiving_on_worker += before_receiving.elapsed();
        //     });
        //     if length == 1 && buffer_a[0] == 42 {
        //         break;
        //     }
        //     std::mem::swap(&mut buffer_a, &mut buffer_b);
        // }

        // Solution three: Asynchronous receiving using tokio
        // let mut receive_task: Option<tokio::task::JoinHandle<Vec<u8>>> = Option::None;
        // loop {
        //     let before_receiving = Instant::now();
        //     // if let Some(task) = receive_task {}
        //     let maybe_output = match receive_task.take() {
        //         Some(input) => {
        //             let vec = input.await.unwrap();
        //             if vec.len() == 1 && vec[0] == 42 {
        //                 break;
        //             }
        //             Some(vec)
        //         }
        //         None => None,
        //     };
        //
        //     receive_task = Some(spawn(async {
        //         let world = SimpleCommunicator::world();
        //         let (data, _status) = world.process_at_rank(0).receive_vec::<u8>();
        //         return data;
        //     }));
        //
        //     let Some(data) = maybe_output else {
        //         continue;
        //     };
        //
        //     time_spend_receiving_on_worker += before_receiving.elapsed();
        //
        //     let before_sorting = Instant::now();
        //     let root = data[0].clone();
        //     let input = u8_to_entries_unsafe(data);
        //     if buffers[root as usize].is_some() {
        //         buffers[root as usize].as_mut().unwrap().join(input);
        //     } else {
        //         buffers[root as usize] = Some(input.into());
        //     }
        //     time_spend_sorting_on_worker += before_sorting.elapsed();
        // }

        // // Solution four: Asynchronous calculation using tokio
        // // Only one thread for sorting
        // // Could be improved by using multiple threads for sorting
        // // That way would probably only be IO bound, if we have enough cores
        // let mut buffers: Arc<Mutex<[Option<SortedEntries>; 256]>> =
        //     Arc::new(Mutex::new(arr_macro::arr![None; 256]));
        // let mut sort_task: Option<tokio::task::JoinHandle<()>> = Option::None;
        // loop {
        //     let before_receiving = Instant::now();
        //     let (data, _status) = world.process_at_rank(0).receive_vec::<u8>();
        //     if data.len() == 1 && data[0] == 42 {
        //         break;
        //     }
        //     time_spend_receiving_on_worker += before_receiving.elapsed();
        //
        //     let before_sorting = Instant::now();
        //     let buffers = buffers.clone();
        //     if let Some(task) = sort_task.take() {
        //         task.await.unwrap();
        //     }
        //     sort_task = Some(spawn(async move {
        //         let before_sorting = Instant::now();
        //         let mut buffers = buffers.lock().await;
        //         let root = data[0].clone();
        //         let input = u8_to_entries_unsafe(data);
        //         if buffers[root as usize].is_some() {
        //             buffers[root as usize].as_mut().unwrap().join(input);
        //         } else {
        //             buffers[root as usize] = Some(input.into());
        //         }
        //         time_spend_sorting_on_worker += before_sorting.elapsed();
        //     }));
        //     time_spend_sorting_on_worker += before_sorting.elapsed();
        // }
        // if let Some(task) = sort_task.take() {
        //     task.await.unwrap();
        // }

        // // Solution four: Asynchronous calculation using tokio
        // // Only one thread for sorting
        // // Could be improved by using multiple threads for sorting
        // // That way would probably only be IO bound, if we have enough cores
        // let mut buffers: std::sync::Arc<std::sync::Mutex<[Option<SortedEntries>; 256]>> =
        //     std::sync::Arc::new(std::sync::Mutex::new(arr_macro::arr![None; 256]));
        // let mut sort_task: Option<std::thread::JoinHandle<()>> = Option::None;
        // loop {
        //     let before_receiving = Instant::now();
        //     let (data, _status) = world.process_at_rank(0).receive_vec::<u8>();
        //     if data.len() == 1 && data[0] == 42 {
        //         break;
        //     }
        //     time_spend_receiving_on_worker += before_receiving.elapsed();
        //
        //     let before_sorting = Instant::now();
        //     let buffers = buffers.clone();
        //     sort_task = Some(std::thread::spawn(move || {
        //         if let Some(task) = sort_task.take() {
        //             task.join().unwrap();
        //         }
        //         let before_sorting = Instant::now();
        //         let mut buffers = buffers.lock().unwrap();
        //         let root = data[0].clone();
        //         let input = u8_to_entries_unsafe(data);
        //         if buffers[root as usize].is_some() {
        //             buffers[root as usize].as_mut().unwrap().join(input);
        //         } else {
        //             buffers[root as usize] = Some(input.into());
        //         }
        //         time_spend_sorting_on_worker += before_sorting.elapsed();
        //     }));
        //     time_spend_sorting_on_worker += before_sorting.elapsed();
        // }
        // if let Some(task) = sort_task.take() {
        //     task.join().unwrap();
        // }
        // let mut buffers = buffers.lock().unwrap();
        // let bufferss = &mut *buffers;
        // let mut buffers = arr_macro::arr![None; 256];
        // std::mem::swap(bufferss, &mut empty_buffers);

        let world = SimpleCommunicator::world();
        let send_start = Instant::now();
        buffers.into_iter().for_each(|buffer| {
            let buffer = buffer.map(|b| b.into_vec()).unwrap_or(Vec::new());
            if buffer.len() == 0 {
                return;
            }
            let result_vec = entries_to_u8_unsafe(buffer);
            world.process_at_rank(0).send(result_vec.as_slice());
        });
        // world.process_at_rank(0).send(&[42u8]);
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
    let metrics = Handle::current().metrics();

    // eprintln!("Node {} has {:?} workers", rank, metrics);
    eprintln!("Node {} has {:?} workers", rank, metrics.num_workers());
    eprintln!(
        "Node {} has {:?} threads",
        rank,
        metrics.num_blocking_threads()
    );
    eprintln!(
        "Node {} has {:?} active tasks",
        rank,
        metrics.active_tasks_count()
    );
    eprintln!(
        "Node {} has {:?} outside scheduled tasks scheduled from outside of the runtime.",
        rank,
        metrics.remote_schedule_count()
    );
    return vec![];
}
