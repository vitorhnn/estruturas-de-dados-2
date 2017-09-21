extern crate byteorder;

use std::fs::OpenOptions;

mod serialize;
mod cliente;
mod hash_table;

use cliente::Cliente;
use hash_table::FileHashTable;

fn main() {
    let hash = OpenOptions::new().write(true).read(true).create(true).open("hash").unwrap();
    let data = OpenOptions::new().write(true).read(true).create(true).open("data").unwrap();
    let mut table = FileHashTable::new(hash, data);

    let c = Cliente {
        codigo: 0,
        nome: "Motocicleberson".to_string(),
    };

    table.insert(c.codigo as u64, c).unwrap();

    let c = Cliente {
        codigo: 1,
        nome: "Clay Town".to_string(),
    };

    table.insert(c.codigo as u64, c.clone()).unwrap();

    table.delete(c.codigo as u64).unwrap();
}
