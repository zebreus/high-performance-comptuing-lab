use arr_macro::arr;
use itertools::Itertools;
use rdst::{RadixKey, RadixSort};

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

pub struct RadixDivider<const BLOCK_SIZE: usize> {
    buffers: [Vec<Entry>; 256],
}

impl<const BLOCK_SIZE: usize> RadixDivider<BLOCK_SIZE> {
    pub fn new() -> Self {
        RadixDivider {
            buffers: arr![{let mut vec = Vec::new(); vec.reserve(BLOCK_SIZE); vec}; 256],
        }
    }

    pub fn push(&mut self, entry: &Entry) {
        let first_byte = entry.key()[0];
        self.buffers[first_byte as usize].push(entry.clone());
    }

    pub fn push_all(&mut self, entries: &[Entry]) {
        for entry in entries {
            let first_byte = entry.key()[0];
            self.buffers[first_byte as usize].push(entry.clone());
        }
    }

    pub fn delegate_buffers(&mut self, delegator: &mut dyn FnMut(u8, Vec<Entry>)) {
        for (index, buffer) in self.buffers.iter_mut().enumerate() {
            if buffer.len() >= BLOCK_SIZE {
                delegator(index as u8, std::mem::take(buffer));
            }
        }
    }

    pub fn delegate_remaining_buffers(&mut self, delegator: &mut dyn FnMut(u8, Vec<Entry>)) {
        for (index, buffer) in self.buffers.iter_mut().enumerate() {
            if buffer.len() > 0 {
                delegator(index as u8, std::mem::take(buffer));
            }
        }
    }
}

pub struct SortedEntries {
    entries: Vec<Entry>,
}

impl Into<SortedEntries> for Vec<Entry> {
    fn into(mut self) -> SortedEntries {
        self.radix_sort_builder()
            .with_single_threaded_tuner()
            .with_parallel(false)
            .sort();
        SortedEntries { entries: self }
    }
}

impl SortedEntries {
    pub fn trust_me_bro_this_is_already_sorted(entries: Vec<Entry>) -> Self {
        SortedEntries { entries }
    }

    pub fn new() -> Self {
        SortedEntries {
            entries: Vec::new(),
        }
    }

    pub fn join(&mut self, other: Self) {
        // let entries = self.entries;
        // TODO: Make this more efficient, as it copies every time, maybe use a list
        let merged_vec = self
            .entries
            .iter()
            .cloned()
            .merge(other.entries.into_iter())
            .collect::<Vec<_>>();

        self.entries = merged_vec;
    }

    pub fn into_vec(self) -> Vec<Entry> {
        self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
