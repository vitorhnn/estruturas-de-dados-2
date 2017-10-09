extern crate byteorder;

use std::fs::File;

mod hash_table;
mod serialize;
mod cliente;

use cliente::Cliente;
use hash_table::InnerLinkingFileHashTable;

fn main() {
    let c = Cliente {
        codigo: 0,
        nome: "AAAAAA".to_string()
    };

    let mut f = File::create("cuin").unwrap();

    let mut table = InnerLinkingFileHashTable::<Cliente>::new(f, 7);

    table.insert(c.codigo as u64, c);


    println!("aaa");
}
