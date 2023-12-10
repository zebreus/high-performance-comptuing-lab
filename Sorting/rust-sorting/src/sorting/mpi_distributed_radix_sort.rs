use core::panic;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::Arc,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use itertools::Itertools;
use mpi::{environment::Universe, request::StaticScope, topology::SimpleCommunicator, traits::*};
use mpi::{request::WaitGuard, Rank};
use rdst::RadixSort;
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt},
    spawn,
    sync::Mutex,
};

use crate::entry::{
    entries_to_u8_unsafe, radix_divide, u8_to_entries_unsafe, Entry, RadixDivider, SortedEntries,
};

/// Size of the blocks of entries that will be transmitted to one worker in one go. In entries/ 100bytes
const BLOCK_SIZE: usize = 10000; // 10000 Entries = 1 MB
/// Size of datablocks that will be read from disk. In bytes
const READ_BLOCK_SIZE: usize = 256 * 100 * 100;

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

macro_rules! mark {
    // This macro takes an expression of type `expr` and prints
    // it as a string along with its result.
    // The `expr` designator is used for expressions.
    ( $name:expr, $preparation:expr   ) => {
        // `stringify!` will convert the expression *as it is* into a string.
        {
            let before = Instant::now();
            let value = $preparation;
            let duration = before.elapsed();
            eprintln!("{} took {} micros.", $name, duration.as_micros());
            value
        }
    };
}

pub fn get_worker(bucket_id: i32, workers: i32) -> Rank {
    return ((bucket_id as i32) % workers) + 1;
}

pub async fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    // let universe = mpi::initialize().unwrap();
    let world = SimpleCommunicator::world();
    let size = world.size();
    let rank = world.rank();

    let mut read_duration = Duration::new(0, 0);
    let mut copy_duration = Duration::new(0, 0);
    let mut processing_duration = Duration::new(0, 0);
    let mut send_duration = Duration::new(0, 0);
    let mut post_duration = Duration::new(0, 0);

    // println!("Hello from process {} of {}", rank, size);

    if rank == 0 {
        // let bytes = Vec::<u8>::new();

        measure_time!("distributing", {
            let file = tokio::fs::File::open(input_file).await.unwrap();

            let mut radix_divider = RadixDivider::<BLOCK_SIZE>::new();

            let mut reader = Arc::new(Mutex::new(file));
            let mut work_box: Arc<Mutex<Box<[u8; READ_BLOCK_SIZE]>>> =
                Arc::new(Mutex::new(Box::new([0; READ_BLOCK_SIZE])));
            let mut read_box: Arc<Mutex<Box<[u8; READ_BLOCK_SIZE]>>> =
                Arc::new(Mutex::new(Box::new([0; READ_BLOCK_SIZE])));
            // let mut my_buffer = &mut *my_box;
            // let mut read_buffer = &mut *read_box;
            let mut buffer_end: usize = 0;
            // buffer.reserve(100 * 256 * BLOCK_SIZE);

            let mut send_task: Option<tokio::task::JoinHandle<()>> = Option::None;
            let mut read_task: Option<tokio::task::JoinHandle<usize>> = Option::None;

            loop {
                let before_read = Instant::now();
                // let length;
                // let before_copy;
                {
                    match read_task {
                        Some(task) => {
                            buffer_end = task.await.unwrap();
                            if buffer_end == 0 {
                                break;
                            }
                        }
                        None => {}
                    }
                }
                read_duration += before_read.elapsed();
                {
                    let reader = reader.clone();
                    let read_box = read_box.clone();
                    let work_box = work_box.clone();

                    read_task = Some(tokio::spawn(async move {
                        let mut current_end = 0;
                        let mut r = reader.lock().await;
                        let mut read_box = read_box.lock().await;
                        let read_buffer = &mut **read_box;
                        while current_end % 100 != 0 || current_end == 0 {
                            let length = r
                                .read_buf(&mut &mut read_buffer[current_end..])
                                .await
                                .unwrap();
                            // eprintln!("Read {} bytes", length);
                            if length == 0 {
                                break;
                            }
                            current_end += length;
                        }

                        let mut target_box = work_box.lock().await;

                        std::mem::swap(&mut *target_box, &mut *read_box);
                        return current_end;
                        // if current_end == 0 {
                        //     break;
                        // }
                    }));
                }
                let before_processing = Instant::now();
                let before_copy = Instant::now();
                let my_work_box = work_box.lock().await;

                // buffer[y](read_buffer);
                // reader.consume(length);

                let my_buffer = &my_work_box[..buffer_end];
                let entries = unsafe {
                    // assert!(buffer_end % 100 == 0);
                    core::slice::from_raw_parts(
                        my_buffer.as_ptr() as *const Entry,
                        buffer_end / 100,
                    )
                };

                copy_duration += before_copy.elapsed();
                // let entries = buffer_end / 100;
                // let real_length = entries * 100;
                // let overhang = buffer_end - real_length;
                // let buffer = &my_buffer[0..real_length];
                // eprintln!("Length: {} , {}", real_length, buffer.len());
                // eprintln!("Overhang: {}", overhang);

                // spawn(reader.fill_buf());

                // let buffer = read_buffer[0..real_length].to_vec();
                // reader.consume(real_length);

                // println!(
                //     "{}: Got buffer of length {} {}",
                //     Instant::now().duration_since(start_time).as_micros(),
                //     length,
                //     real_length
                // );

                // if length == 0 {
                //     break;
                // }
                // let entries = unsafe {
                //     core::slice::from_raw_parts(buffer.as_ptr() as *const Entry, buffer.len() / 100)
                // };

                radix_divider.push_all(entries);

                processing_duration += before_processing.elapsed();
                let before_send = Instant::now();

                let all_buffers_ready = radix_divider.are_all_buffers_ready();
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
                send_duration += before_send.elapsed();
                let before_post = Instant::now();

                // if overhang > 0 {
                //     let (first, second) = my_buffer.split_at_mut(overhang);
                //     first.copy_from_slice(
                //         &second[(real_length - overhang)..(buffer_end - overhang)],
                //     );
                // }
                // buffer_end = overhang;
                post_duration += before_post.elapsed();
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
            send_duration += before_send.elapsed();

            for i in 0..(size - 1) {
                world.process_at_rank(i + 1).send(&[42u8]);
            }
        });
        // Now we have distributed that stuff, lets try to get the sorted data back

        let mut writer_thread: Option<tokio::task::JoinHandle<()>> = Option::None;
        let output_file_path = output_directory.join("output.sorted");
        let output_file = std::fs::File::create(&output_file_path).unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(output_file)));

        for bucket_id in 0..=255 {
            let node = get_worker(bucket_id, size - 1);
            let (data, _status) = world.process_at_rank(node).receive_vec::<u8>();
            if data.len() == 1 && data[0] == 42 {
                panic!("Got done from node {} when expecting a result", node);
            }
            if let Some(thread) = writer_thread.take() {
                thread.await.unwrap();
            }
            let writer = Arc::clone(&writer);
            writer_thread = Some(spawn(async move {
                let mut writer = writer.lock().await;
                writer.write_all(&data).unwrap();
            }));
        }

        if let Some(thread) = writer_thread.take() {
            thread.await.unwrap();
        }
        writer.lock().await.flush().unwrap();
    }

    if rank != 0 {
        // let mut buffers: Vec<Vec<Entry>> = Vec::new();
        let mut buffers: [Option<SortedEntries>; 256] = arr_macro::arr![None; 256];
        // let mut buffers: [Vec<Entry>; 256] = arr_macro::arr![Vec::new(); 256];
        measure_time!("node receiving", {
            // let buffer = SortedEntries::new();
            let mut counter = 0;
            loop {
                let (data, _status) = world.process_at_rank(0).receive_vec::<u8>();
                counter = counter + 1;
                // eprintln!(
                //     "{}: Process {} got message {}.",
                //     Instant::now().duration_since(start_time).as_micros(),
                //     rank,
                //     counter
                // );
                if data.len() == 1 && data[0] == 42 {
                    break;
                }
                let root = data[0].clone();
                let input = u8_to_entries_unsafe(data);
                // input
                //     .radix_sort_builder()
                //     .with_single_threaded_tuner()
                //     .with_parallel(false)
                //     .sort();
                // glidesort::sort(&mut input);
                // buffers.push(input);

                // if buffers[root as usize].is_some() {
                //     buffers[root as usize].as_mut().and_then(|buffer| {
                //         Some(buffer.join(SortedEntries::trust_me_bro_this_is_already_sorted(input)))
                //     });
                // } else {
                //     buffers[root as usize] =
                //         Some(SortedEntries::trust_me_bro_this_is_already_sorted(input));
                // }

                // input.sort_unstable();
                // buffers[root as usize].append(&mut input);
                if buffers[root as usize].is_some() {
                    buffers[root as usize].as_mut().unwrap().join(input);
                } else {
                    buffers[root as usize] = Some(input.into());
                }
            }
        });

        // eprintln!("Process {} got {} entries.", rank, buffer.len());
        // vecc.radix_sort_builder()
        //     .with_single_threaded_tuner()
        //     .with_parallel(false)
        //     .sort();

        // let temp: &[Entry] = &vecc.clone()[0..10];
        // let debug_buffers = &temp
        //     .into_iter()
        //     .map(|entry| entry.key())
        //     .collect::<Vec<_>>();
        // println!("BUFFERS: {:?}", debug_buffers)
        // measure_time!("actually sorting", {
        //     buffers.iter_mut().for_each(|buffer| {
        //         // buffer
        //         //     .radix_sort_builder()
        //         //     .with_single_threaded_tuner()
        //         //     .with_parallel(false)
        //         //     .sort();
        //         // glidesort::sort(buffer);
        //         buffer.sort_unstable();
        //     });
        // });
        buffers.into_iter().for_each(|buffer| {
            let buffer = buffer.map(|b| b.into_vec()).unwrap_or(Vec::new());
            if buffer.len() == 0 {
                return;
            }
            let result_vec = entries_to_u8_unsafe(buffer);
            let root = result_vec[0];
            // eprintln!("Process {} repling with batch {}.", rank, root);
            world.process_at_rank(0).send(result_vec.as_slice());
        });
        world.process_at_rank(0).send(&[42u8]);

        // measure_time!("node sending", {
        //     let result_chunks = result_vec.chunks(BLOCK_SIZE * 100);
        //     let mut result_counter = 0;
        //     for chunk in result_chunks {
        //         result_counter += 1;
        //         eprintln!("Process {} repling chunk {}.", rank, result_counter);
        //         world.process_at_rank(0).send(chunk);
        //     }
        //     world.process_at_rank(0).send(&[42u8]);
        // });
    }

    // match rank {
    //     0 => {
    //         let msg = vec![4.0f64, 8.0, 15.0];
    //         world.process_at_rank(rank + 1).send(&msg[..]);
    //     }
    //     1 => {
    //         let (msg, status) = world.any_process().receive_vec::<f64>();
    //         println!(
    //             "Process {} got message {:?}.\nStatus is: {:?}",
    //             rank, msg, status
    //         );
    //     }
    //     _ => unreachable!(),
    // }
    if rank == 0 {
        print!(
            "{},{},{},{},{},",
            read_duration.as_secs_f64(),
            copy_duration.as_secs_f64(),
            processing_duration.as_secs_f64(),
            send_duration.as_secs_f64(),
            post_duration.as_secs_f32()
        );
        eprintln!("Read took {} micros.", read_duration.as_micros());
        eprintln!("Copy took {} micros.", copy_duration.as_micros());
        eprintln!(
            "Processing took {} micros.",
            processing_duration.as_micros()
        );
        eprintln!("Send took {} micros.", send_duration.as_micros());
        eprintln!("Post took {} micros.", post_duration.as_micros());
    }
    return vec![];
}
