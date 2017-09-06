extern crate byteorder;
extern crate chrono;

mod serializable;
mod cliente;

use std::fs::File;
use std::cmp::min;

use serializable::Serializable;
use serializable::SerializeError;

use cliente::Cliente;

use chrono::prelude::*;

struct Node<T> where T: PartialOrd + Clone {
    val: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T> where T: PartialOrd + Clone {
    fn new_leaf(val: T) -> Node<T> {
        Node {
            val,
            left: None,
            right: None,
        }
    }

    fn choose_winner(left: Node<T>, right: Node<T>) -> Node<T> {
        if left.val < right.val {
            Node {
                val: left.val.clone(),
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            }
        } else {
            Node {
                val: right.val.clone(),
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
            }
        }
    }

    fn construct(values: Vec<T>) -> Node<T> {
        let mut iteration: Vec<Node<T>> = values.into_iter().map(|x| Node::new_leaf(x)).collect();
        let mut next_iteration = Vec::with_capacity(iteration.len() / 2);

        while iteration.len() > 1 {
            while !iteration.is_empty() {
                let right = iteration.pop().unwrap();
                let maybe_left = iteration.pop();

                let parent = match maybe_left {
                    Some(left) => Node::choose_winner(left, right),
                    None => right,
                };

                next_iteration.push(parent);
            }

            iteration = next_iteration;
            next_iteration = Vec::with_capacity(iteration.len() / 2);
        }

        iteration.pop().unwrap()
    }
}

fn nway_merge<T>(mut files: Vec<File>) -> Result<(), SerializeError> where T: PartialOrd + Clone + Serializable {
    // no more than 4 files opened at a time.

    let cutoff = min(3, files.len());
    let mut split = files.split_off(cutoff); // split 4 files

    let mut vals = Vec::with_capacity(files.len());

    for mut file in files {
        vals.push(T::deserialize(&mut file)?);
    }

    let winner = Node::construct(vals);

    Ok(())
}

fn main() {
    nway_merge::<Cliente>(vec![File::open("bucket-0").unwrap(), File::open("bucket-1").unwrap()]);

    println!("aeiou");
}

