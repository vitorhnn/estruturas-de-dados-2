use std::fs::File;
use std::marker;
use std::mem::size_of;
use std::io::SeekFrom;

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
    next: i64,
    valid: bool,
}

impl<T> Record<T> where T: Serialize {
    fn from_t(val: T) -> Record<T> {
        Record {
            val,
            next: -1,
            valid: true,
        }
    }
}

impl<T> Serialize for Record<T> where T: Serialize {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError> {
        self.val.serialize(file);
        file.write_i64::<BigEndian>(self.next);

        let write = if self.valid == false {
            0
        } else {
            1
        };

        file.write_u32::<BigEndian>(write);
        Ok(())
    }

    fn deserialize(file: &mut File) -> Result<Self, SerializeError> {
        let val = T::deserialize(file)?;
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
            next,
            valid
        })
    }
}

impl<T> FileHashTable<T> where T: Serialize {
    fn new(table: File, entries: File) -> FileHashTable<T> {
        let size = 7;
        table.set_len((4 * size_of::<usize>()) as u64).unwrap();
        FileHashTable{ table, entries, size, _marker: marker::PhantomData }
    }

    fn insert(&self, key: i64, val: T) {
        let hash = key % (self.size as i64); // this is not really a proper hash function
        self.table.seek(SeekFrom::Start(hash * size_of::<usize>()));
    }
}

