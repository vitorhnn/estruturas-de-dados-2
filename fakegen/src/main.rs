extern crate byteorder;
extern crate chrono;
extern crate rand;

use std::fs::File;
//use std::collections::HashSet;

use rand::Rng;
use chrono::prelude::*;

mod cliente;
mod serializable;

use serializable::Serializable;
use cliente::Cliente;

fn main() {
    let mut records = Vec::with_capacity(10000);
    let mut rng = rand::thread_rng();
    for _ in 0..10000 {
        records.push(
            Cliente {
                codigo: rng.gen::<u32>(),
                nome: "Nemo".to_string(),
                data_nascimento: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(rng.gen_range(0, Utc::now().timestamp()), 0), Utc),
            }
        );
    }

    for i in 0..records.len() / 400 {
        let mut bucket = File::create(format!("bucket-{}", i)).expect("Failed to create a bucket");

        let len = records.len().saturating_sub(400);

        let mut tail = records.split_off(len);

        tail.sort();

        for record in tail {
            record.serialize(&mut bucket).expect("Failed to write to a bucket");
        }
    }
}
