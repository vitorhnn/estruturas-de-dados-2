// keys are again limited to i64 for academic reasons

use std::marker;

use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::mem::size_of;

use serialize::{SerializeError, Serialize, PackedSize};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

pub struct InnerLinkingFileHashTable<T: Serialize + PackedSize> {
    table: File,
    size: usize,
    _marker: marker::PhantomData<T>,
}

struct Record<T: Serialize + PackedSize> {
    val: T,
    key: u64,
    next: i64,
    valid: bool,
}

impl<T> PackedSize for Record<T> where T: Serialize + PackedSize {
    fn packed_size() -> usize {
        return T::packed_size() + 64 + 64 + size_of::<bool>();
    }
}

impl<T> Record<T> where T: Serialize + PackedSize {
    fn from_t(val: T, key: u64) -> Record<T> {
        Record {
            val,
            key,
            next: -1,
            valid: true,
        }
    }
}

impl<T> Serialize for Record<T> where T: Serialize + PackedSize {
    fn serialize(&self, file: &mut Write) -> Result<(), SerializeError> {
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

    fn deserialize(file: &mut Read) -> Result<Self, SerializeError> {
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

impl<T> InnerLinkingFileHashTable<T> where T: Serialize + PackedSize { // we java now
    pub fn open(table: File, size: usize) -> InnerLinkingFileHashTable<T> {
        InnerLinkingFileHashTable { table, size, _marker: marker::PhantomData }
    }

    pub fn new(mut table: File, size: usize) -> InnerLinkingFileHashTable<T> {
        let dummy: Vec<u8> = vec![0; Record::<T>::packed_size()];

        for _ in 0..size {
            table.write_all(&dummy).unwrap();
        }

        InnerLinkingFileHashTable::<T>::open(table, size)
    }

    fn search_for_empty(&mut self) -> Result<u64, SerializeError> {
        let mut offset = self.table.seek(SeekFrom::Start(0))?;
        let mut record = Record::<T>::deserialize(&mut self.table)?;

        while record.valid {
            offset = self.table.seek(SeekFrom::Current(0))?;
            record = match Record::<T>::deserialize(&mut self.table) {
                Ok(val) => val,
                Err(_) => return Ok(self.table.seek(SeekFrom::End(0))?),
            }
        }

        Ok(offset)
    }

    pub fn insert(&mut self, key: u64, val: T) -> Result<(), SerializeError> {
        let hash = key % (self.size as u64);
        let pos = hash * Record::<T>::packed_size() as u64;
        self.table.seek(SeekFrom::Start(pos));

        let mut entry = Record::<T>::deserialize(&mut self.table)?;

        if !entry.valid {
            self.table.seek(SeekFrom::Start(pos));

            let entry = Record::from_t(val, key);
            entry.serialize(&mut self.table)?;
        } else {
            let mut prev_offset = pos;
            while entry.next != -1 {
                prev_offset = self.table.seek(SeekFrom::Start(entry.next as u64))?;
                entry = Record::<T>::deserialize(&mut self.table)?;
            }

            let write_at = self.search_for_empty()?;
            self.table.seek(SeekFrom::Start(write_at))?; // this is probably not needed
            Record::from_t(val, key).serialize(&mut self.table)?;
            self.table.seek(SeekFrom::Start(prev_offset))?;
            entry.next = write_at as i64;
            entry.serialize(&mut self.table)?;
        }

        Ok(())
    }
}