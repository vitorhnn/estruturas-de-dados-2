use std::fs::File;
use std::marker;
use std::mem::size_of;
use std::io::{Seek, SeekFrom};

// keys are limited to i64s for academic reasons

use serialize::{Serialize, SerializeError};

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

pub struct FileHashTable<T> where T: Serialize {
    table: File, // TODO: Cursors, maybe?
    entries: File,
    size: usize,
    _marker: marker::PhantomData<T>,
}

pub struct Record<T: Serialize> {
    val: T,
    key: u64,
    next: i64,
    valid: bool,
}

impl<T> Record<T> where T: Serialize {
    fn from_t(val: T, key: u64) -> Record<T> {
        Record {
            val,
            key,
            next: -1,
            valid: true,
        }
    }
}

impl<T> Serialize for Record<T> where T: Serialize {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError> {
        self.val.serialize(file)?;
        file.write_u64::<BigEndian>(self.key)?;
        file.write_i64::<BigEndian>(self.next)?;

        let write = if self.valid == false {
            0
        } else {
            1
        };

        file.write_u32::<BigEndian>(write)?;
        Ok(())
    }

    fn deserialize(file: &mut File) -> Result<Self, SerializeError> {
        let val = T::deserialize(file)?;
        let key = file.read_u64::<BigEndian>()?;
        let next = file.read_i64::<BigEndian>()?;
        let valid: bool;

        let intval = file.read_u32::<BigEndian>()?;

        if intval == 0 {
            valid = false;
        } else {
            valid = true;
        }

        Ok(Record {
            val,
            key,
            next,
            valid
        })
    }
}

impl<T> FileHashTable<T> where T: Serialize {
    pub fn open(table: File, entries: File, size: usize) -> FileHashTable<T> {
        FileHashTable{ table, entries, size, _marker: marker::PhantomData }
    }

    pub fn new(mut table: File, entries: File, size: usize) -> FileHashTable<T> {
        for _ in 0..size {
            table.write_i64::<BigEndian>(-1).unwrap();
        }

        FileHashTable::<T>::open(table, entries, size)
    }

    fn search_for_empty(&mut self) -> Result<u64, SerializeError> {
        let mut offset = self.entries.seek(SeekFrom::Start(0))?;
        let mut record = Record::<T>::deserialize(&mut self.entries)?;

        while record.valid {
            offset = self.entries.seek(SeekFrom::Current(0))?; // this is probably butt slow
            record = Record::<T>::deserialize(&mut self.entries)?;
        }

        Ok(offset)
    }

    pub fn insert(&mut self, key: u64, val: T) -> Result<(), SerializeError> {
        let hash = key % (self.size as u64); // this is not really a proper hash function
        let pos = hash * size_of::<usize>() as u64;
        self.table.seek(SeekFrom::Start(pos))?;
        let maybe_offset = self.table.read_i64::<BigEndian>()?;

        if maybe_offset == -1 {
            let written_at = match self.search_for_empty() {
                Ok(val) => self.entries.seek(SeekFrom::Start(val))?,
                Err(_) => self.entries.seek(SeekFrom::End(0))?,
            };

            let record = Record::from_t(val, key);
            record.serialize(&mut self.entries)?;
            self.table.seek(SeekFrom::Start(pos))?;
            self.table.write_u64::<BigEndian>(written_at)?;
        } else {
            let mut entry_offset = self.entries.seek(SeekFrom::Start(maybe_offset as u64))?;
            let mut record = Record::<T>::deserialize(&mut self.entries)?;

            while record.next != -1 && record.key != key {
                entry_offset = self.entries.seek(SeekFrom::Start(record.next as u64))?;
                record = Record::<T>::deserialize(&mut self.entries)?;
            }

            if record.key == key {
                // already present
                return Ok(());
            }

            let written_at = match self.search_for_empty() {
                Ok(val) => val,
                Err(_) => self.entries.seek(SeekFrom::End(0))?
            };

            let new_record = Record::from_t(val, key);
            new_record.serialize(&mut self.entries)?;
            record.next = written_at as i64;
            self.entries.seek(SeekFrom::Start(entry_offset))?;
            record.serialize(&mut self.entries)?;
        }

        Ok(())
    }

    pub fn delete(&mut self, key: u64) -> Result<(), SerializeError> {
        let hash = key % (self.size as u64);
        let pos = hash * size_of::<usize>() as u64;
        self.table.seek(SeekFrom::Start(pos))?;
        let maybe_offset = self.table.read_i64::<BigEndian>()?;

        if maybe_offset == -1 {
            panic!("Attempted to delete an unregistered key");
        } else {
            let mut cur_offset = self.entries.seek(SeekFrom::Start(maybe_offset as u64))?;
            let mut record = Record::<T>::deserialize(&mut self.entries)?;

            let mut prev_offset = cur_offset;

            while record.key != key {
                prev_offset = cur_offset;
                cur_offset = self.entries.seek(SeekFrom::Start(record.next as u64))?;
                record = Record::<T>::deserialize(&mut self.entries)?;
            }

            if prev_offset == cur_offset {
                // jumped from the hash table to the buckets.
                self.entries.seek(SeekFrom::Start(cur_offset as u64))?;
                record.valid = false;
                record.next = -1;
                record.serialize(&mut self.entries)?;

                self.table.seek(SeekFrom::Start(pos))?;
                self.table.write_i64::<BigEndian>(-1)?;
            } else {
                self.entries.seek(SeekFrom::Start(cur_offset as u64))?;
                record.valid = false;
                let next = record.next;
                record.next = -1;
                record.serialize(&mut self.entries)?;

                self.entries.seek(SeekFrom::Start(prev_offset as u64))?;
                let mut record = Record::<T>::deserialize(&mut self.entries)?;
                self.entries.seek(SeekFrom::Start(prev_offset as u64))?;
                record.next = next;
                record.serialize(&mut self.entries)?;
            }

            Ok(())
        }
    }

    pub fn search(&mut self, key: u64) -> Result<T, SerializeError> {
        let hash = key % (self.size as u64);
        let pos = hash * size_of::<usize>() as u64;
        self.table.seek(SeekFrom::Start(pos))?;
        let maybe_offset = self.table.read_i64::<BigEndian>()?;

        if maybe_offset == -1 {
            panic!("Attempted to search an unregistered key"); // TODO: maybe not panic
        } else {
            self.entries.seek(SeekFrom::Start(maybe_offset as u64))?;
            let mut maybe_return = Record::<T>::deserialize(&mut self.entries)?;

            while maybe_return.key != key  {
                if maybe_return.next != -1 {
                    self.entries.seek(SeekFrom::Start(maybe_return.next as u64))?;
                    maybe_return = Record::<T>::deserialize(&mut self.entries)?;
                } else {
                    panic!("Key not registered"); // again, this should probably not be a panic
                }
            }

            Ok(maybe_return.val)
        }
    }
}

