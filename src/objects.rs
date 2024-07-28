use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex_literal::hex;
use sha1::{Digest, Sha1};
use std::io;
use std::io::prelude::*;

use crate::repository::Repository;

struct Commit {
    data: Vec<u8>,
    size: usize,
}

struct Blob {
    data: Vec<u8>,
    size: usize,
}

struct Tree {
    data: Vec<u8>,
    size: usize,
}

struct Tag {
    data: Vec<u8>,
    size: usize,
}

pub enum Object {
    Commit(Commit),
    Blob(Blob),
    Tree(Tree),
    Tag(Tag),
}

impl Object {
    pub fn get_data(&self) -> &Vec<u8> {
        match self {
            Object::Commit(c) => &c.data,
            Object::Blob(b) => &b.data,
            Object::Tree(t) => &t.data,
            Object::Tag(t) => &t.data,
        }
    }

    pub fn get_type(&self) -> &str {
        match self {
            Object::Commit(_) => "commit",
            Object::Blob(_) => "blob",
            Object::Tree(_) => "tree",
            Object::Tag(_) => "tag",
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            Object::Commit(c) => c.size,
            Object::Blob(b) => b.size,
            Object::Tree(t) => t.size,
            Object::Tag(t) => t.size,
        }
    }

    pub fn deserialize(repository: &Repository, sha: &str) -> Self {
        let path = repository.get_object_path(sha);
        let data = Self::read(&path);

        // the data consists of the object type, a space 0x20, the object size, a null byte 0x00, and the object content

        let mut data = data.as_slice();
        let mut space = data.iter().position(|&x| x == 0x20).unwrap();
        let obj_type = &data[0..space];
        let obj_type = std::str::from_utf8(obj_type).expect("Invalid Object Type");

        data = &data[space + 1..];
        let mut null = data.iter().position(|&x| x == 0x00).unwrap();
        let size = &data[0..null];
        let size = std::str::from_utf8(size).expect("Invalid Object Size");

        data = &data[null + 1..];
        let size = size.parse::<usize>().expect("Failed to parse object size");
        let content = &data[0..size];

        match obj_type {
            "commit" => {
                let commit = Commit {
                    data: content.to_vec(),
                    size,
                };
                Object::Commit(commit)
            }
            "blob" => {
                let blob = Blob {
                    data: content.to_vec(),
                    size,
                };
                Object::Blob(blob)
            }
            "tree" => {
                let tree = Tree {
                    data: content.to_vec(),
                    size,
                };
                Object::Tree(tree)
            }
            "tag" => {
                let tag = Tag {
                    data: content.to_vec(),
                    size,
                };
                Object::Tag(tag)
            }
            _ => unimplemented!("Non Known Object Type"),
        }
    }

    fn read(path: &str) -> Vec<u8> {
        let data = std::fs::read(path).expect("Failed to read object file");

        let mut z = ZlibDecoder::new(&data[..]);
        let mut s = Vec::new();
        //println!("{:?}", z.into_inner().read_to_string(buf));
        z.read_to_end(&mut s).expect("Failed to decompress object");

        s
    }

    pub fn serialize(&self) {
        unimplemented!()
    }
}
