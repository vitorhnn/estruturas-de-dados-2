extern crate byteorder;
extern crate chrono;

mod serializable;
mod cliente;

use std::fs::File;
use std::cmp::{min, Ordering};

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

#[derive(Clone)]
struct TWrapper<T> where T: PartialOrd {
    val: Option<T>,
    file_idx: usize,
}

impl<T> PartialEq for TWrapper<T>
    where T: PartialOrd
{
    fn eq(&self, other: &TWrapper<T>) -> bool{
        self.val == other.val
    }
}

impl<T> PartialOrd for TWrapper<T>
where T: PartialOrd
{
    fn partial_cmp(&self, other: &TWrapper<T>) -> Option<Ordering> {
        self.val.partial_cmp(&other.val)
    }
}

fn replace_root<T>(root: &mut Node<TWrapper<T>>, new_val: Option<T>)
    where T: PartialOrd + Clone
{
    let mut path = Vec::new();
    let node = root;
    path.push(node);
    let is_leaf = false;

    while !is_leaf {
        if let Some(ref mut branch) = node.left {
            if branch.val.file_idx == node.val.file_idx {
                node = branch;
            }
        }
    }
}

fn nway_merge<T>(mut files: Vec<File>) -> Result<(), SerializeError> where T: PartialOrd + Clone + Serializable {
    // no more than 4 files opened at a time.

    let mut iteration = 0;
    let mut output = File::create(format!("merge-{}", iteration))?;

    let cutoff = min(3, files.len());
    let mut split = files.split_off(cutoff); // split 4 files

    let mut vals = Vec::with_capacity(files.len());

    for (idx, mut file) in files.iter_mut().enumerate() {
        let val = T::deserialize(&mut file)?;
        vals.push(TWrapper { val: Some(val) , file_idx: idx });
    }

    let mut winner = Node::construct(vals);

    // TODO: get rid of this clone
    winner.val.val.clone().unwrap().serialize(&mut output)?;

    let insert = match T::deserialize(&mut files[winner.val.file_idx]) {
        Ok(val) => Some(val),
        _ => None,
    };

    replace_root(&mut winner, insert);

    Ok(())
}

fn main() {
    /*
    let a = Cliente { codigo: 1000, nome: "aaa".to_string(), data_nascimento: Utc::now() };

    let mut f = File::create("bucket-0").unwrap();

    a.serialize(&mut f);

    let a = Cliente { codigo: 1001, nome: "aaa".to_string(), data_nascimento: Utc::now() };

//    a.serialize(&mut f);

    let mut f = File::create("bucket-1").unwrap();

    let a = Cliente { codigo: 1002, nome: "aaa".to_string(), data_nascimento: Utc::now() };

    a.serialize(&mut f);
    */

    nway_merge::<Cliente>(vec![File::open("bucket-0").unwrap(), File::open("bucket-1").unwrap()]);

    println!("aeiou");
}

