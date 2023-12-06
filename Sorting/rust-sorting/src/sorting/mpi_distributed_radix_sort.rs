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
    io::{AsyncBufRead, AsyncBufReadExt},
    spawn,
    sync::Mutex,
};

use crate::entry::{
    entries_to_u8_unsafe, radix_divide, u8_to_entries_unsafe, Entry, RadixDivider, SortedEntries,
};

const BLOCK_SIZE: usize = 10000; // 1 MB

macro_rules! measure_time {
    // This macro takes an expression of type `expr` and prints
    // it as a string along with its result.
    // The `expr` designator is used for expressions.
    ( $name:expr, $preparation:block   ) => {
        // `stringify!` will convert the expression *as it is* into a string.
        let before = Instant::now();
        $preparation
        let duration = before.elapsed();
        println!(
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
            println!("{} took {} micros.", $name, duration.as_micros());
            value
        }
    };
}

pub fn get_worker(bucket_id: i32, workers: i32) -> Rank {
    return ((bucket_id as i32) % workers) + 1;
}

pub async fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    let universe = mpi::initialize().unwrap();
    let world = SimpleCommunicator::world();
    spawn(async move {
        let mut file = File::create("test.txt").unwrap();
        file.write_all(b"Hello, world!").unwrap();
        let world = SimpleCommunicator::world();
        world.any_process();
    });
    let size = world.size();
    let rank = world.rank();
    let start_time = Instant::now();

    let mut read_duration = Duration::new(0, 0);
    let mut copy_duration = Duration::new(0, 0);
    let mut processing_duration = Duration::new(0, 0);
    let mut send_duration = Duration::new(0, 0);
    let mut post_duration = Duration::new(0, 0);

    println!("Hello from process {} of {}", rank, size);

    if rank == 0 {
        // let bytes = Vec::<u8>::new();

        measure_time!("distributing", {
            let file = tokio::fs::File::open(input_file).await.unwrap();

            let mut radix_divider = RadixDivider::<BLOCK_SIZE>::new();

            let mut reader = Arc::new(Mutex::new(tokio::io::BufReader::with_capacity(
                100 * 256 * BLOCK_SIZE * 10,
                file,
            )));
            let mut my_box: Box<[u8; 100 * 256 * BLOCK_SIZE]> =
                Box::new([0; 100 * 256 * BLOCK_SIZE]);
            let mut my_buffer = &mut *my_box;
            let mut buffer_start: usize = 0;
            let mut buffer_end: usize = 0;
            // buffer.reserve(100 * 256 * BLOCK_SIZE);

            let mut send_task: Option<tokio::task::JoinHandle<()>> = Option::None;

            loop {
                let before_read = Instant::now();
                let length;
                let before_copy;
                {
                    let mut r = reader.lock().await;
                    let read_buffer = mark!("fill_buf", r.fill_buf().await.unwrap());
                    read_duration += before_read.elapsed();
                    before_copy = Instant::now();
                    length = read_buffer.len();
                    my_buffer[buffer_end..(buffer_end + length)].copy_from_slice(read_buffer);
                    buffer_end += length;
                    r.consume(length);
                }
                {
                    let r = reader.clone();
                    let handle = tokio::spawn(async move {
                        eprintln!("Awating buffer.");
                        r.lock().await.fill_buf().await;
                        eprintln!("Awated buffer.");
                    });
                    // handle.await.unwrap();
                }

                // buffer[y](read_buffer);
                // reader.consume(length);

                let entries = buffer_end / 100;
                let real_length = entries * 100;
                let overhang = buffer_end - real_length;
                let buffer = &my_buffer[0..real_length];

                // spawn(reader.fill_buf());

                // let buffer = read_buffer[0..real_length].to_vec();
                // reader.consume(real_length);
                copy_duration += before_copy.elapsed();

                println!(
                    "{}: Got buffer of length {} {}",
                    Instant::now().duration_since(start_time).as_micros(),
                    length,
                    real_length
                );

                let before_processing = Instant::now();

                if length == 0 {
                    break;
                }
                let entries = unsafe {
                    core::slice::from_raw_parts(buffer.as_ptr() as *const Entry, buffer.len() / 100)
                };

                radix_divider.push_all(entries);

                processing_duration += before_processing.elapsed();
                let all_buffers_ready = radix_divider.are_all_buffers_ready();
                let before_send = Instant::now();

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
                                if block.len() == 0 {
                                    return;
                                }
                                let target = get_worker(*root as i32, size - 1);

                                // eprintln!("Sending message to process {}", target + 1);
                                guards.push(WaitGuard::from(
                                    world.process_at_rank(target).immediate_send(scope, block),
                                ));
                            }
                            measure_time!("wait for guard", {
                                guards.clear();
                            });
                        });
                    }));
                    // handle.await.unwrap();
                }
                send_duration += before_send.elapsed();
                let before_post = Instant::now();

                if overhang > 0 {
                    let (first, second) = my_buffer.split_at_mut(overhang);
                    first.copy_from_slice(
                        &second[(real_length - overhang)..(buffer_end - overhang)],
                    );
                }
                buffer_end = overhang;
                post_duration += before_post.elapsed();
            }
            if let Some(send_task) = send_task.take() {
                send_task.await.unwrap();
            }
            let before_send = Instant::now();
            let full_buffers = radix_divider.get_remaining_buffers();
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
            let mut buffer = SortedEntries::new();
            let mut counter = 0;
            loop {
                let (data, _status) = world.process_at_rank(0).receive_vec::<u8>();
                counter = counter + 1;
                eprintln!(
                    "{}: Process {} got message {}.",
                    Instant::now().duration_since(start_time).as_micros(),
                    rank,
                    counter
                );
                if data.len() == 1 && data[0] == 42 {
                    break;
                }
                let root = data[0].clone();
                let mut input = u8_to_entries_unsafe(data);
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
            eprintln!("Process {} repling with batch {}.", rank, root);
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
        println!("Read took {} micros.", read_duration.as_micros());
        println!("Copy took {} micros.", copy_duration.as_micros());
        println!(
            "Processing took {} micros.",
            processing_duration.as_micros()
        );
        println!("Send took {} micros.", send_duration.as_micros());
        println!("Post took {} micros.", post_duration.as_micros());
    }
    return vec![];
}
