// TODO: all of this is horrible and I should be ashamed

extern crate byteorder;
extern crate chrono;

mod serializable;
mod cliente;

use std::fs::File;
use std::cmp::Ordering;

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

#[derive(Debug, Clone, Copy)]
enum Side {
    Left,
    Right,
}

// credit where it is due:
// https://stackoverflow.com/questions/36167160/recursive-data-structures-in-rust
struct NodeZipper<T> where T: PartialOrd + Clone {
    node: Node<T>,
    parent: Option<Box<NodeZipper<T>>>,
    side: Side,
}

impl<T> NodeZipper<T> where T: PartialOrd + Clone {
    fn child(mut self, side: Side) -> NodeZipper<T> {
        let child = match side {
            Side::Left => *self.node.left.take().unwrap(),
            Side::Right => *self.node.right.take().unwrap(),
        };

        NodeZipper {
            node: child,
            parent: Some(Box::new(self)),
            side
        }
    }

    fn parent(self) -> NodeZipper<T> {
        let NodeZipper { node, parent, side } = self;

        let NodeZipper {
            node: mut parent_node,
            parent: parent_parent,
            side: parent_side,
        } = *parent.unwrap();

        match side {
            Side::Left => parent_node.left = Some(Box::new(node)),
            Side::Right => parent_node.right = Some(Box::new(node)),
        }

        NodeZipper {
            node: parent_node,
            parent: parent_parent,
            side: parent_side
        }
    }

    fn finish(mut self) -> Node<T> {
        while let Some(_) = self.parent {
            self = self.parent();
        }

        self.node
    }
}

#[derive(Clone)]
struct TWrapper<T> where T: PartialOrd {
    wrap: T,
    file_idx: usize,
}

impl<T> PartialEq for TWrapper<T>
    where T: PartialOrd
{
    fn eq(&self, other: &TWrapper<T>) -> bool {
        self.wrap == other.wrap
    }
}

impl<T> PartialOrd for TWrapper<T>
    where T: PartialOrd
{
    fn partial_cmp(&self, other: &TWrapper<T>) -> Option<Ordering> {
        self.wrap.partial_cmp(&other.wrap)
    }
}

fn replace_root<T>(root: Node<TWrapper<T>>, new_val: Option<T>) -> Option<Node<TWrapper<T>>>
    where T: PartialOrd + Clone
{
    let mut zipper = NodeZipper { node: root, parent: None, side: Side::Left };
    let mut is_leaf = false;

    // really ugly descent logic
    while !is_leaf {
        let next = if let Some(ref branch) = zipper.node.left {
            if branch.val.file_idx == zipper.node.val.file_idx {
                Some(Side::Left)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(side) = next {
            zipper = zipper.child(side);
        } else {
            let next = if let Some(ref branch) = zipper.node.right {
                if branch.val.file_idx == zipper.node.val.file_idx {
                    Some(Side::Right)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(side) = next {
                zipper = zipper.child(side)
            } else {
                is_leaf = true;
            }
        }
    }

    fn update_node<T: PartialOrd + Clone>(node: &mut Node<T>) {
        // unwrapping here because we're pruning empty branches.
        // also yes, this is not very pretty
        if node.left.as_ref().unwrap().val < node.right.as_ref().unwrap().val {
            node.val = node.left.as_ref().unwrap().val.clone();
        } else {
            node.val = node.right.as_ref().unwrap().val.clone();
        }
    }

    match new_val {
        Some(val) => {
            zipper.node.val.wrap = val;

            while let Some(_) = zipper.parent {
                zipper = zipper.parent();
                update_node(&mut zipper.node);
            }
        }
        None => {
            // check the opposite side, if there's something there, move it and throw the node
            // away.
            // if there isn't, jump to our parent and repeat the process
            // once we finish this, just update the nodes as usual
            // special (?) case: one node tree being replaced by empty value
            if let None = zipper.parent { return None; }

            loop {
                // save our side and go to parent
                let side = zipper.side;
                zipper = zipper.parent();
                match side {
                    Side::Left => {
                        match zipper.node.right.take() {
                            Some(val) => {
                                zipper.node = *val;
                                break;
                            }
                            None => {}
                        }
                    }
                    Side::Right => {
                        match zipper.node.left.take() {
                            Some(val) => {
                                zipper.node = *val;
                                break;
                            }
                            None => {}
                        }
                    }
                }
            }

            while let Some(_) = zipper.parent {
                zipper = zipper.parent();
                update_node(&mut zipper.node);
            }
        }
    }

    Some(zipper.finish())
}

fn nway_merge<T>(mut files: Vec<File>, n: usize) -> Result<(), SerializeError>
    where T: PartialOrd + Clone + Serializable
{
    let mut iteration = 0;
    while files.len() > 1 {
        {
            let mut output = File::create(format!("merge-{}", iteration))?;

            let len = files.len().saturating_sub(n - 1);

            let mut tail = files.split_off(len);

            let mut wrappers = Vec::with_capacity(tail.len());

            for (idx, mut file) in tail.iter_mut().enumerate() {
                let val = T::deserialize(&mut file)?;
                wrappers.push(TWrapper { wrap: val, file_idx: idx });
            }

            let mut winner = Node::construct(wrappers);

            loop {
                winner.val.wrap.serialize(&mut output)?;

                let insert = match T::deserialize(&mut tail[winner.val.file_idx]) {
                    Ok(val) => Some(val),
                    _ => None,
                };

                let replaced = replace_root(winner, insert);

                match replaced {
                    Some(val) => winner = val,
                    None => break,
                }
            }
        }

        files.push(File::open(format!("merge-{}", iteration))?);
        iteration += 1;
    }

    Ok(())
}

fn main() {
    let mut codigo = 0;
    let mut bucket = 0;

    // TODO: Write a mock generator (using Faker?)
    while codigo < 4000 {
        let mut written = 0;
        let mut file = File::create(format!("bucket-{}", bucket)).unwrap();
        while written < 500 {
            Cliente { codigo, nome: "aaa".to_string(), data_nascimento: Utc::now() }.serialize(&mut file);
            codigo += 1;
            written += 1;
        }
        bucket += 1;
    }

    let mut files = Vec::new();

    for i in 0..8 {
        files.push(File::open(format!("bucket-{}", i)).unwrap());
    }

    nway_merge::<Cliente>(files, 12);

    println!("aeiou");
}
