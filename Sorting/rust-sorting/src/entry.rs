pub struct Entry {
    key: [u8; 10],
    data: [u8; 90],
}

impl Into<Entry> for &[u8; 100] {
    fn into(self) -> Entry {
        let mut key = [0u8; 10];
        key.copy_from_slice(&self[0..10]);
        let mut data = [0u8; 90];
        data.copy_from_slice(&self[10..100]);
        Entry { key, data }
    }
}

impl Into<[u8; 100]> for Entry {
    fn into(self) -> [u8; 100] {
        let mut result = [0u8; 100];
        result[0..10].copy_from_slice(&self.key);
        result[10..100].copy_from_slice(&self.data);
        result
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
