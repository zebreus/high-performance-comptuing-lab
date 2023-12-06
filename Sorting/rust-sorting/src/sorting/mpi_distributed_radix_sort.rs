use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use itertools::Itertools;
use mpi::traits::*;
use mpi::Rank;
use rdst::RadixSort;

use crate::entry::{
    entries_to_u8_unsafe, radix_divide, u8_to_entries_unsafe, Entry, RadixDivider, SortedEntries,
};

const BLOCK_SIZE: usize = 10000 * 100; // 1 MB

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
            let mut radix_divider = RadixDivider::<BLOCK_SIZE>::new();
            let mut reader = BufReader::with_capacity(((size as usize) - 1) * BLOCK_SIZE, file);

            let delegator = &mut |root: u8, block: Vec<Entry>| {
                if block.len() == 0 {
                    return;
                }
                let target = (root as i32) % (size - 1);
                let u8_block = entries_to_u8_unsafe(block);

                // eprintln!("Sending message to process {}", target + 1);
                world.process_at_rank(target + 1).send(u8_block.as_slice());
            };

            loop {
                let buffer = reader.fill_buf().unwrap();
                let length = buffer.len();

                if length == 0 {
                    break;
                }
                let entries = unsafe {
                    core::slice::from_raw_parts(buffer.as_ptr() as *const Entry, buffer.len() / 100)
                };

                radix_divider.push_all(entries);

                radix_divider.delegate_buffers(delegator);

                reader.consume(length);
            }
            radix_divider.delegate_remaining_buffers(delegator);

            for i in 0..(size - 1) {
                world.process_at_rank(i + 1).send(&[42u8]);
            }
        });
        // Now we have distributed that stuff, lets try to get the sorted data back

        let mut finished_nodes: Vec<bool> = vec![false; size as usize - 1];
        let mut sorted_buffers: [Vec<Entry>; 256] = arr_macro::arr![Vec::new(); 256];

        loop {
            for node in 1..size {
                if finished_nodes[node as usize - 1] {
                    continue;
                }
                let (data, _status) = world.process_at_rank(node).receive_vec::<u8>();
                // counter = counter + 1;
                // eprintln!("Process {} got message {}.", rank, counter);
                if data.len() == 1 && data[0] == 42 {
                    finished_nodes[node as usize - 1] = true;
                    break;
                }
                let root = data[0].clone();
                let response = u8_to_entries_unsafe(data);
                sorted_buffers[root as usize] = response;
                // let sorted_input: SortedEntries = input.into();
                // buffer = buffer.join(sorted_input);
            }
            if finished_nodes.iter().all(|finished| *finished) {
                break;
            }
        }

        let result: Vec<Entry> = sorted_buffers.into_iter().flatten().collect_vec();

        // let mut buffers: Vec<Option<Vec<Entry>>> = Vec::new();
        // // let workers: usize = (rank - 1).try_into().unwrap();
        // for _ in 0..(size - 1) {
        //     buffers.push(Some(Vec::new()));
        // }

        // let mut result: Vec<Entry> = Vec::new();

        // measure_time!("collecting", {
        //     '_inner: loop {
        //         // Try to refill buffers
        //         for i in 0..(size - 1) {
        //             let index: usize = i.try_into().unwrap();
        //             if let Some(buffer) = &mut buffers[index] {
        //                 if buffer.len() != 0 {
        //                     continue;
        //                 }
        //                 // eprintln!("Master node is waiting for data from process {}.", i + 1);
        //                 let (data, _status) = world.process_at_rank(i + 1).receive_vec::<u8>();
        //                 if data.len() == 1 && data[0] == 42 {
        //                     buffers[index] = None;
        //                     continue;
        //                 }
        //                 buffers[index] = Some(u8_to_entries_unsafe(data));
        //             }
        //         }

        //         let done = buffers.iter().all(|buffer| buffer.is_none());
        //         if done {
        //             break;
        //         }
        //         // Merge buffers until one buffer runs out
        //         let mut available_buffers = buffers
        //             .iter_mut()
        //             .filter_map(|buffer| match buffer {
        //                 Some(buffer) => Some(buffer),
        //                 None => None,
        //             })
        //             .collect::<Vec<_>>();

        //         loop {
        //             let first_entry = available_buffers[0].get(0);

        //             let Some(mut smallest_value) = first_entry else {
        //                 break;
        //             };
        //             let mut smallest_index: usize = 0;
        //             for (index, buffer) in available_buffers.iter().enumerate() {
        //                 if buffer.len() == 0 {
        //                     break;
        //                 }

        //                 if &buffer[0] < smallest_value {
        //                     smallest_value = &buffer[0];
        //                     smallest_index = index;
        //                 }
        //             }
        //             result.push(smallest_value.clone());
        //             available_buffers[smallest_index].remove(0);
        //             if available_buffers[smallest_index].len() == 0 {
        //                 break;
        //             }
        //         }
        //         // let temp = &result[0..10];
        //         // let debug_buffers = &temp
        //         //     .into_iter()
        //         //     .map(|entry| entry.key())
        //         //     .collect::<Vec<_>>();
        //         // // Check if we are done
        //         // println!("BUFFERS: {:?}", debug_buffers);
        //     }
        // });
        let result_bytes = entries_to_u8_unsafe(result);

        measure_time!("writing", {
            let output_file_path = output_directory.join("output.sorted");
            let output_file = std::fs::File::create(&output_file_path).unwrap();
            let mut writer = BufWriter::new(output_file);
            writer.write_all(&result_bytes).unwrap();
            writer.flush().unwrap();
        });

        // let write_duration = before_write.elapsed();
    }

    if rank != 0 {
        // let mut buffers: Vec<Vec<Entry>> = Vec::new();
        let mut buffers: [Option<SortedEntries>; 256] = arr_macro::arr![None; 256];
        let mut buffers: [Vec<Entry>; 256] = arr_macro::arr![Vec::new(); 256];
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
                let root = data[0].clone();
                let mut input = u8_to_entries_unsafe(data);
                // input
                //     .radix_sort_builder()
                //     .with_single_threaded_tuner()
                //     .with_parallel(false)
                //     .sort();
                // glidesort::sort(&mut input);
                input.sort_unstable();
                // buffers.push(input);

                // if buffers[root as usize].is_some() {
                //     buffers[root as usize].as_mut().and_then(|buffer| {
                //         Some(buffer.join(SortedEntries::trust_me_bro_this_is_already_sorted(input)))
                //     });
                // } else {
                //     buffers[root as usize] =
                //         Some(SortedEntries::trust_me_bro_this_is_already_sorted(input));
                // }

                buffers[root as usize].append(&mut input);
                // if .is_some() {
                //     buffers[root as usize].unwrap().append(other)
                // } else {
                //     buffers[root as usize] =
                //         Some(SortedEntries::trust_me_bro_this_is_already_sorted(input));
                // }
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
        measure_time!("actually sorting", {
            buffers.iter_mut().for_each(|buffer| {
                // buffer
                //     .radix_sort_builder()
                //     .with_single_threaded_tuner()
                //     .with_parallel(false)
                //     .sort();
                // glidesort::sort(buffer);
                buffer.sort_unstable();
            });
        });
        buffers.into_iter().for_each(|buffer| {
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
    return vec![];
}
