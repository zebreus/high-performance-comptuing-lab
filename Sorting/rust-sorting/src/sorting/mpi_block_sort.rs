use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use mpi::traits::*;
use mpi::Rank;
use rdst::RadixSort;

use crate::entry::{entries_to_u8_unsafe, u8_to_entries_unsafe, Entry, SortedEntries};

const BLOCK_SIZE: usize = 10000; // 1 MB

macro_rules! measure_time {
    // This macro takes an expression of type `expr` and prints
    // it as a string along with its result.
    // The `expr` designator is used for expressions.
    ( $name:expr, { $($preparation:stmt )*  } ) => {
        // `stringify!` will convert the expression *as it is* into a string.
        let before = Instant::now();
        $($preparation ;)*
        let duration = before.elapsed();
        println!(
            "Finished {} in {} seconds.",
            stringify!($name),
            duration.as_secs_f64()
        );
    };
}
pub fn sort(input_file: &Path, output_directory: &Path) -> Vec<PathBuf> {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    println!("Hello from process {} of {}", rank, size);

    if rank == 0 {
        // let bytes = Vec::<u8>::new();

        measure_time!("distributing", {
            let file = File::open(input_file).unwrap();
            let mut reader = BufReader::with_capacity(100 * BLOCK_SIZE, file);

            let mut receiver: Rank = 0;
            loop {
                let length = {
                    let buffer = reader.fill_buf().unwrap();
                    // Send the buffer to the next process
                    // println!("Sending message to process {}", receiver + 1);
                    world.process_at_rank(receiver + 1).send(&buffer[..]);
                    receiver = (receiver + 1) % (size - 1);

                    buffer.len()
                };
                if length == 0 {
                    break;
                }
                reader.consume(length);
            }

            for i in 0..(size - 1) {
                world.process_at_rank(i + 1).send(&[42u8]);
            }
        });
        // Now we have distributed that stuff, lets try to get the sorted data back

        let mut buffers: Vec<Option<Vec<Entry>>> = Vec::new();
        // let workers: usize = (rank - 1).try_into().unwrap();
        for _ in 0..(size - 1) {
            buffers.push(Some(Vec::new()));
        }

        let mut result: Vec<Entry> = Vec::new();

        measure_time!("collecting", {
            '_inner: loop {
                // Try to refill buffers
                for i in 0..(size - 1) {
                    let index: usize = i.try_into().unwrap();
                    if let Some(buffer) = &mut buffers[index] {
                        if buffer.len() != 0 {
                            continue;
                        }
                        // eprintln!("Master node is waiting for data from process {}.", i + 1);
                        let (data, _status) = world.process_at_rank(i + 1).receive_vec::<u8>();
                        if data.len() == 1 && data[0] == 42 {
                            buffers[index] = None;
                            continue;
                        }
                        buffers[index] = Some(u8_to_entries_unsafe(data));
                    }
                }

                let done = buffers.iter().all(|buffer| buffer.is_none());
                if done {
                    break;
                }
                // Merge buffers until one buffer runs out
                let mut available_buffers = buffers
                    .iter_mut()
                    .filter_map(|buffer| match buffer {
                        Some(buffer) => Some(buffer),
                        None => None,
                    })
                    .collect::<Vec<_>>();

                loop {
                    let first_entry = available_buffers[0].get(0);

                    let Some(mut smallest_value) = first_entry else {
                        break;
                    };
                    let mut smallest_index: usize = 0;
                    for (index, buffer) in available_buffers.iter().enumerate() {
                        if buffer.len() == 0 {
                            break;
                        }

                        if &buffer[0] < smallest_value {
                            smallest_value = &buffer[0];
                            smallest_index = index;
                        }
                    }
                    result.push(smallest_value.clone());
                    available_buffers[smallest_index].remove(0);
                    if available_buffers[smallest_index].len() == 0 {
                        break;
                    }
                }
                // let temp = &result[0..10];
                // let debug_buffers = &temp
                //     .into_iter()
                //     .map(|entry| entry.key())
                //     .collect::<Vec<_>>();
                // // Check if we are done
                // println!("BUFFERS: {:?}", debug_buffers);
            }
        });

        measure_time!("writing", {
            let output_file_path = output_directory.join("output.sorted");
            let output_file = std::fs::File::create(&output_file_path).unwrap();
            let mut writer = BufWriter::new(output_file);
            let result_bytes = entries_to_u8_unsafe(result);
            writer.write_all(&result_bytes).unwrap();

            writer.flush().unwrap();
        });

        // let write_duration = before_write.elapsed();
    }

    if rank != 0 {
        measure_time!("node receiving", {
            let mut buffer = SortedEntries::new();
            let mut counter = 0;
            loop {
                let (data, _status) = world.process_at_rank(0).receive_vec::<u8>();
                counter = counter + 1;
                eprintln!("Process {} got message {}.", rank, counter);
                if data.len() == 1 && data[0] == 42 {
                    break;
                }
                let input = u8_to_entries_unsafe(data);
                let sorted_input: SortedEntries = input.into();
                buffer.join(sorted_input);
            }
        });

        // eprintln!("Process {} got {} entries.", rank, buffer.len());
        let vecc = buffer.into_vec();
        // vecc.radix_sort_builder()
        //     .with_single_threaded_tuner()
        //     .with_parallel(false)
        //     .sort();

        // let temp: &[Entry] = &vecc.clone()[0..10];
        // let debug_buffers = &temp
        //     .into_iter()
        //     .map(|entry| entry.key())
        //     .collect::<Vec<_>>();
        // println!("BUFFERS: {:?}", debug_buffers);

        measure_time!("node sending", {
            let result_vec = entries_to_u8_unsafe(vecc);

            let result_chunks = result_vec.chunks(BLOCK_SIZE * 100);
            // let mut result_counter = 0;
            for chunk in result_chunks {
                // result_counter += 1;
                // eprintln!(
                //     "Process {} repling chunk {} of {}.",
                //     rank, result_counter, counter
                // );
                world.process_at_rank(0).send(chunk);
            }
            world.process_at_rank(0).send(&[42u8]);
        });
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
    return vec![];
}
