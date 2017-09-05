extern crate byteorder;

use std::fs::File;
use std::mem;

mod agencia;

use agencia::Agencia;
use agencia::Serializable;

enum FreezeWrapper<T> {
    Inactive,
    Active(T),
    // Option is here because I need to replace Frozens with Actives
    // and the borrow checker requires me to put something in Frozen
    Frozen(Option<T>),
}

use FreezeWrapper::*;

fn find<T: PartialOrd>(vec: &Vec<FreezeWrapper<T>>) -> usize {
    let mut idx = 0;

    for (i, entry) in vec.iter().enumerate() {
        let cmp = &vec[idx];

        match *cmp {
            Inactive => idx = i,
            Frozen(_) => idx = i,
            Active(ref y) => match *entry {
                Inactive => (),
                Frozen(_) => (),
                Active(ref z) => if *z < *y { idx = i } else { () },
            }
        }
    }

    idx
}

fn get_new<T>(mut input: &mut File, min: &T) -> FreezeWrapper<T> where T: Serializable + PartialOrd {
    let new_element = T::deserialize(input);

    let new_element = match new_element {
        Ok(x) => x,
        Err(_) => return Inactive,
    };

    if new_element < *min {
        Frozen(Some(new_element))
    } else {
        Active(new_element)
    }
}

fn unfreeze_vec<T>(vec: &mut Vec<FreezeWrapper<T>>) {
    for element in vec {
        *element = if let FreezeWrapper::Frozen(ref mut val) = *element {
            FreezeWrapper::Active(mem::replace(val, None).unwrap())
        } else {
            continue;
        }
    }
}

fn replacement<T>(mut input: &mut File) -> Result<(), agencia::SerializeError> where T: Serializable + PartialOrd {
    let mut vec: Vec<FreezeWrapper<T>> = Vec::with_capacity(9);

    for _ in 0..7 {
        vec.push(Active(T::deserialize(input).unwrap()));
    }

    let mut current_bucket = 0;

    let mut done = false;

    while !done {
        let mut bucket = File::create(format!("bucket-{}", current_bucket))?;

        loop {
            let min_idx = find(&vec);

            let new_val = match vec[min_idx] {
                Active(ref val) => {
                    val.serialize(&mut bucket)?;
                    Some(get_new(input, val))
                },
                Frozen(_) => {
                    None
                },
                Inactive => {
                    done = true;
                    break;
                },
            };

            match new_val {
                Some(val) => {
                    vec.push(val);
                    vec.swap_remove(min_idx);
                },
                None => {
                    unfreeze_vec(&mut vec);
                    break;
                }
            }
        }

        current_bucket += 1;
    }

    Ok(())
}

fn main() {
    let mut f = File::open("db").unwrap();

    replacement::<Agencia>(&mut f).unwrap();
}

