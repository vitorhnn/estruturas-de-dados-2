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

    let mut table = if hash.metadata().unwrap().len() > 0 && data.metadata().unwrap().len() > 0 {
        FileHashTable::open(hash, data, 7)
    } else {
        FileHashTable::new(hash, data, 7)
    };

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

    let moto = table.search(0).unwrap();

    println!("{}", moto.nome);
}
