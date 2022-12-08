use crate::{StringObject, Value};

#[derive(Debug)]
pub struct Table<'a> {
    pub entries: Vec<Entry<'a>>,
    pub count: usize,
    pub capacity: usize,
}
#[derive(Debug, PartialEq)]
pub struct Entry<'a> {
    pub key: Option<StringObject<'a>>,
    pub value: Value,
}

impl<'a> Entry<'a> {
    pub fn new() -> Self {
        Self {
            key: None,
            value: Value::nil(),
        }
    }
}

const MAX_LOAD: f64 = 0.75;

fn find_entry_slot<'a>(entries: &Vec<Entry<'a>>, capacity: usize, key: &StringObject<'a>) -> usize {
    let mut index = key.hash as usize % capacity;

    let mut tombstone: Option<usize> = None;

    loop {
        let entry = &entries[index];

        if entry.key.is_none() {
            if entry.value.is_nil() {
                return if tombstone.is_some() {
                    tombstone.unwrap()
                } else {
                    index
                };
            } else {
                if tombstone.is_none() {
                    tombstone = Some(index);
                }
            }
        } else if entry.key.as_ref() == Some(key) {
            return index;
        }

        index = (index + 1) % capacity;
    }
}

impl<'a> Table<'a> {
    pub fn new() -> Self {
        Self {
            entries: vec![],
            count: 0,
            capacity: 0,
        }
    }

    pub fn set(&mut self, key: StringObject<'a>, value: Value) -> bool {
        if (self.count + 1) as f64 > self.capacity as f64 * MAX_LOAD {
            self.adjust_capacity(if self.capacity < 8 {
                8
            } else {
                self.capacity * 2
            });
        }

        let slot = find_entry_slot(&self.entries, self.capacity, &key);

        let mut entry = self.entries.get_mut(slot).unwrap();

        let is_new_key = entry.key.is_none();

        if is_new_key && entry.value.is_nil() {
            self.count += 1;
        }

        entry.key = Some(key);
        entry.value = value;

        is_new_key
    }

    pub fn get(&self, key: &StringObject<'a>) -> Option<Value> {
        if self.count == 0 {
            return None;
        }

        let slot = find_entry_slot(&self.entries, self.capacity, key);

        let entry = self.entries.get(slot).unwrap();

        if entry.key.is_none() {
            return None;
        }

        Some(entry.value)
    }

    pub fn delete(&mut self, key: &StringObject<'a>) -> bool {
        if self.count == 0 {
            return false;
        }

        let slot = find_entry_slot(&self.entries, self.capacity, key);

        let entry = self.entries.get_mut(slot).unwrap();

        if entry.key.is_none() {
            return false;
        }

        // Place a tombstone in the entry.
        entry.key = None;
        entry.value = Value::bool(false);

        self.count -= 1;

        true
    }

    pub fn add_all(&mut self, other: &mut Table<'a>) {
        let other_entries = std::mem::replace(&mut other.entries, vec![]);
        for entry in other_entries {
            if entry.key.is_none() {
                continue;
            }

            self.set(entry.key.unwrap(), entry.value);
        }
    }

    fn adjust_capacity(&mut self, new_capacity: usize) {
        let mut new_entries = Vec::with_capacity(new_capacity);

        let mut old_entries = std::mem::replace(&mut self.entries, vec![]);

        for _ in 0..new_capacity {
            new_entries.push(Entry::new());
        }

        self.count = 0;

        for _ in 0..self.capacity {
            old_entries.drain(..).for_each(|entry| {
                if entry.key.is_none() {
                    return;
                }

                let dest = find_entry_slot(&new_entries, new_capacity, entry.key.as_ref().unwrap());

                new_entries[dest] = entry;

                self.count += 1;
            });
        }

        self.capacity = new_capacity;
        self.entries = new_entries;
    }
}
