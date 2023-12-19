use std::{cmp::Ordering, collections::BinaryHeap};

use arr_macro::arr;
use rdst::RadixKey;

use crate::sorting::BLOCK_SIZE;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    key: [u8; 10],
    data: [u8; 90],
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            key: [0u8; 10],
            data: [0u8; 90],
        }
    }
}

impl Entry {
    pub fn new(key: [u8; 10], data: [u8; 90]) -> Self {
        Entry { key, data }
    }

    pub fn key(&self) -> &[u8; 10] {
        &self.key
    }

    pub fn bucket(&self) -> usize {
        self.key[0] as usize
    }

    pub fn data(&self) -> &[u8; 90] {
        &self.data
    }
}

// impl Into<Entry> for &[u8; 100] {
//     fn into(self) -> Entry {
//         let mut key = [0u8; 10];
//         key.copy_from_slice(&self[0..10]);
//         let mut data = [0u8; 90];
//         data.copy_from_slice(&self[10..100]);
//         Entry { key, data }
//     }
// }

impl<'a> Into<&'a Entry> for &'a [u8; 100] {
    fn into(self) -> &'a Entry {
        unsafe { std::mem::transmute(self) }
    }
}

impl Into<Entry> for [u8; 100] {
    fn into(self) -> Entry {
        unsafe { std::mem::transmute(self) }
    }
}

pub fn u8_to_entries_unsafe(mut vec8: Vec<u8>) -> Vec<Entry> {
    // I copy-pasted this code from StackOverflow without reading the answer
    // surrounding it that told me to write a comment explaining why this code
    // is actually safe for my own use case.
    // Yes, I did!
    let vec_entry = unsafe {
        // let ratio = std::mem::size_of::<u8>() / std::mem::size_of::<Entry>();
        assert!(vec8.len() % 100 == 0, "vec8.len() % 100 == 0");
        assert!(vec8.capacity() % 100 == 0, "vec8.capacity() % 100 == 0");

        let length = vec8.len() / 100;
        let capacity = vec8.capacity() / 100;
        let ptr = vec8.as_mut_ptr() as *mut Entry;

        // Don't run the destructor for vec32
        std::mem::forget(vec8);

        // Construct new Vec
        Vec::from_raw_parts(ptr, length, capacity)
    };

    return vec_entry;
}

// // Convert a byte slice ref into a entry slice ref without copying
// // This is safe, because the memory layout of Entry is the same as [u8; 100]
// #[inline]
// pub fn u8_to_entries(mut bytes: &[u8]) -> &[Entry] {
//     unsafe {
//         let length = bytes.len();
//         assert!(length % 100 == 0);
//         core::slice::from_raw_parts(bytes.as_ptr() as *const Entry, length / 100)
//     }
// }

// Convert a byte slice ref into a entry slice ref without copying
// This is safe, because the memory layout of Entry is the same as [u8; 100]
#[inline]
pub fn boxed_u8_to_entries(bytes: Box<[u8]>, length: usize) -> Vec<Entry> {
    unsafe {
        let capacity = bytes.len();
        assert!(length % 100 == 0);
        assert!(capacity >= length);

        let pointer = (*Box::into_raw(bytes)).as_mut_ptr() as *mut Entry;

        Vec::from_raw_parts(pointer, length / 100, capacity / 100)
    }
}

pub fn entries_to_u8_unsafe(mut vec_entry: Vec<Entry>) -> Vec<u8> {
    // I copy-pasted this code from StackOverflow without reading the answer
    // surrounding it that told me to write a comment explaining why this code
    // is actually safe for my own use case.
    // Yes, I did!
    let vec_entry = unsafe {
        let length = vec_entry.len() * 100;
        let capacity = vec_entry.capacity() * 100;
        let ptr = vec_entry.as_mut_ptr() as *mut u8;

        // Don't run the destructor for vec32
        std::mem::forget(vec_entry);

        // Construct new Vec
        Vec::from_raw_parts(ptr, length, capacity)
    };

    return vec_entry;
}

// impl Into<[u8; 100]> for Entry {
//     fn into(self) -> [u8; 100] {
//         let mut result = [0u8; 100];
//         result[0..10].copy_from_slice(&self.key);
//         result[10..100].copy_from_slice(&self.data);
//         result
//     }
// }

impl<'a> Into<&'a [u8; 100]> for &'a Entry {
    fn into(self) -> &'a [u8; 100] {
        unsafe { std::mem::transmute(self) }
    }
}

impl Into<[u8; 100]> for Entry {
    fn into(self) -> [u8; 100] {
        unsafe { std::mem::transmute(self) }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl RadixKey for Entry {
    const LEVELS: usize = 10;

    #[inline]
    fn get_level(&self, level: usize) -> u8 {
        self.key[Self::LEVELS - 1 - level]
    }
}

// pub fn sort_entries(entries: &mut Vec<Entry>) {
//     let entries: [Option<Vec<Entry>>; 256] = [Option::None {}; 256];
// }

pub fn radix_divide<'a, Iter: Iterator<Item = &'a Entry>>(
    entries: Iter,
    delegator: &mut dyn FnMut(u8, Vec<Entry>),
) {
    let mut buffers: [Vec<Entry>; 256] = arr![vec![]; 256];

    for entry in entries {
        let first_byte = entry.key()[0];
        buffers[first_byte as usize].push(entry.clone());
    }

    for (index, buffer) in buffers.into_iter().enumerate() {
        delegator(index as u8, buffer);
    }
}

pub struct RadixDivider {
    buffers: Box<[Entry; 256 * 2 * BLOCK_SIZE]>,
    positions: [usize; 256],
    // buffer_slices: [&'static mut [Entry; 2 * BLOCK_SIZE]; 256],
}

impl RadixDivider {
    pub fn new() -> Self {
        let mut positions = [0; 256];
        for i in 0..256 {
            positions[i] = i * BLOCK_SIZE * 2;
        }

        RadixDivider {
            // buffers: vec![Entry::default(); 256 * 2 * BLOCK_SIZE].into_boxed_slice(),
            buffers: unsafe { Box::new_zeroed().assume_init() },
            positions,
        }
    }

    #[inline]
    fn push(&mut self, entry: &Entry) {
        let key = entry.key()[0];
        let index = self.positions[key as usize];
        self.buffers[index] = entry.clone();
        self.positions[key as usize] += 1;
    }

    pub fn push_all(&mut self, entries: &[Entry]) {
        for entry in entries {
            self.push(entry);
        }
    }

    /// Extracts all buckets
    pub fn get_delegateable_buffers(&mut self) -> Vec<(usize, Vec<u8>)> {
        let mut result = Vec::new();
        for (index, block_end) in self.positions.iter_mut().enumerate() {
            let block_start = index * BLOCK_SIZE * 2;
            let sl = entries_to_u8_unsafe(self.buffers[(block_start)..(*block_end)].to_vec());
            *block_end = block_start;
            result.push((index, sl));
        }
        result
    }

    /// Borrows all buckets
    pub fn borrow_delegateable_buffers(&mut self) -> Vec<(usize, &[u8])> {
        let result = self
            .buffers
            .chunks_exact(2 * BLOCK_SIZE)
            .enumerate()
            .map(|(index, buffer)| {
                let end = self.positions[index];
                let start = 2 * BLOCK_SIZE * index;
                let length = end - start;
                let buffer = &buffer[0..length];
                let u8buffer = unsafe {
                    core::slice::from_raw_parts(buffer.as_ptr() as *const u8, length * 100)
                };
                (index, u8buffer)
            })
            .collect::<Vec<_>>();
        for i in 0..256 {
            self.positions[i] = i * BLOCK_SIZE * 2;
        }
        return result;
    }

    /// Check if all buckets are filled with more than BLOCK_SIZE elements
    pub fn ready_to_delegate(&mut self) -> bool {
        !self.positions.iter().any(|position| position < &BLOCK_SIZE)
    }
}

#[derive(Debug)]
pub struct SortedEntries {
    entries: Vec<Entry>,
    others: Vec<Vec<Entry>>,
}

impl Into<SortedEntries> for Vec<Entry> {
    fn into(mut self) -> SortedEntries {
        self.sort_unstable();
        SortedEntries {
            entries: self,
            others: Vec::new(),
        }
    }
}

fn merge_sorted_vecs(vecs: Vec<Vec<Entry>>) -> Vec<Entry> {
    let new_length: usize = vecs.iter().map(|vec| vec.len()).sum();
    let mut min_heap: BinaryHeap<MinElement> = vecs
        .into_iter()
        .filter(|v| !v.is_empty())
        .map(|v| MinElement { vec: v, index: 0 })
        .collect();

    let mut result = Vec::with_capacity(new_length);

    while let Some(mut min_element) = min_heap.pop() {
        result.push(min_element.vec[min_element.index].clone());

        min_element.index += 1;
        if min_element.index < min_element.vec.len() {
            // eprintln!("Pushing {:?}", min_element.index);
            min_heap.push(min_element);
        }
    }

    result
}

// Wrapper struct to store the minimum element along with its source vector and index
#[derive(Debug, Eq, PartialEq)]
struct MinElement {
    vec: Vec<Entry>,
    index: usize,
}

// Implement Ord and PartialOrd manually to make MinElement usable in BinaryHeap
impl Ord for MinElement {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering to create a min heap
        other.vec[other.index].cmp(&self.vec[self.index])
        // self.vec[self.index].cmp(&other.vec[other.index])
    }
}

impl PartialOrd for MinElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Merge once other gets this much bigger than main
const GROWTH_FACTOR: usize = 1;

impl SortedEntries {
    pub fn trust_me_bro_this_is_already_sorted(entries: Vec<Entry>) -> Self {
        SortedEntries {
            entries,
            others: Vec::new(),
        }
    }

    pub fn new() -> Self {
        SortedEntries {
            entries: Vec::new(),
            others: Vec::new(),
        }
    }

    pub fn join(&mut self, mut other: Vec<Entry>) {
        other.sort_unstable();
        self.others.push(other);

        // let entries = self.entries;
        // TODO: Make this more efficient, as it copies every time, maybe use a list
        let main_length = self.entries.len();
        let other_length: usize = self.others.iter().map(|vec| vec.len()).sum();

        if (GROWTH_FACTOR * other_length) > main_length {
            self.actually_join();
        }
    }

    pub fn join_slice(&mut self, mut other: Vec<Entry>) {
        other.sort_unstable();
        self.others.push(other);

        // let entries = self.entries;
        // TODO: Make this more efficient, as it copies every time, maybe use a list
        let main_length = self.entries.len();
        let other_length: usize = self.others.iter().map(|vec| vec.len()).sum();

        if (GROWTH_FACTOR * other_length) > main_length {
            self.actually_join();
        }
    }

    pub fn actually_join(&mut self) {
        let other_length: usize = self.others.iter().map(|vec| vec.len()).sum();
        if other_length == 0 {
            return;
        }

        let mut all = std::mem::take(&mut self.others);
        all.push(std::mem::take(&mut self.entries));
        self.entries = merge_sorted_vecs(all);
    }

    pub fn into_vec(mut self) -> Vec<Entry> {
        self.actually_join();
        // eprintln!(
        //     "{:?}",
        //     &self.entries[0..10]
        //         .iter()
        //         .map(|entry| entry.key())
        //         .collect::<Vec<_>>()
        // );
        self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
