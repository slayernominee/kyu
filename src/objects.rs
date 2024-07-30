use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::{collections::HashMap, io::prelude::*};

use crate::{repository::Repository, ObjectType};

#[derive(Clone)]
pub struct Commit {
    data: Vec<u8>,
    size: usize,
}

pub struct Blob {
    data: Vec<u8>,
    size: usize,
}

impl Blob {
    pub fn from_file(path: &str) -> Object {
        let data = std::fs::read(path).expect("Couldnt read file");
        let size = data.len();

        let blob = Blob { data, size };

        Object::Blob(blob)
    }
}

pub struct Tree {
    data: Vec<u8>,
    size: usize,
    objects: Vec<TreeEntry>,
}

pub struct TreeEntry {
    mode: String,
    name: String,
    sha: String,
    object: Object,
}

impl TreeEntry {
    pub fn get_object(&self) -> &Object {
        &self.object
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_hash(&self) -> &str {
        &self.sha
    }
}

struct Tag {
    data: Vec<u8>,
    size: usize,
}

impl Commit {
    pub fn hash(&self) -> String {
        Object::Commit(self.clone()).hash()
    }
}

pub trait KVLM {
    fn get_data(&self) -> &Vec<u8>;

    fn get_parents(&self) -> Vec<String> {
        let (_, parents) = self.to_kvlm();
        parents
    }

    fn get_tree(&self) -> String {
        let (kvlm, _) = self.to_kvlm();
        kvlm.get("tree").expect("No tree").to_string()
    }

    fn get_message(&self) -> String {
        let (kvlm, _) = self.to_kvlm();
        kvlm.get("message").expect("No message").to_string()
    }

    fn get_author(&self) -> String {
        let (kvlm, _) = self.to_kvlm();
        kvlm.get("author").expect("No author").to_string()
    }

    fn to_kvlm(&self) -> (HashMap<String, String>, Vec<String>) {
        let data = std::str::from_utf8(self.get_data()).expect("Invalid utf8 data");
        let data = data.replace("\n ", "");

        let message = data.split("\n\n").collect::<Vec<&str>>()[1];

        let data = data.split("\n\n").collect::<Vec<&str>>()[0];

        let mut parents = vec![];

        let mut kvlm = data
            .split("\n")
            .map(|line| {
                let mut parts = line.split(' ');
                let key = parts.next().expect("No key");
                let value = parts.collect::<Vec<&str>>().join(" ");

                if key == "parent" {
                    parents.push(value.to_string());

                    // dont continue
                    return (String::new(), String::new());
                }
                (key.to_string(), value.to_string())
            })
            .collect::<HashMap<String, String>>();

        kvlm.insert("message".to_string(), message.to_string());

        (kvlm, parents)
    }
}

impl KVLM for Commit {
    fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl KVLM for Tag {
    fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

pub enum Object {
    Commit(Commit),
    Blob(Blob),
    Tree(Tree),
    Tag(Tag),
}

impl Tree {
    pub fn get_objects(&self) -> &Vec<TreeEntry> {
        &self.objects
    }

    pub fn checkout(&self, path: String, path_to_checkout: String) {
        // the paths differ somewhere and are not subfolders of each other
        if !path_to_checkout.contains(&path)
            && !path.contains(&path_to_checkout)
            && path_to_checkout != path
        {
            return;
        }

        for object in self.objects.iter() {
            match object.get_object() {
                Object::Tree(t) => {
                    t.checkout(
                        path.clone() + "/" + object.get_name(),
                        path_to_checkout.clone(),
                    );
                }
                Object::Blob(b) => {
                    let p = path.clone() + "/" + object.get_name();
                    if !p.contains(&path_to_checkout) {
                        continue;
                    } else {
                        println!("should checkout: {}/{}", path, object.get_name());
                    }
                }
                _ => unimplemented!("Not implemented"),
            }
        }
    }

    pub fn display_objects(&self) -> String {
        let mut result = String::new();
        for object in self.objects.iter() {
            result.push_str(&format!("{} {} {}\n", object.mode, object.sha, object.name));
        }
        result
    }

    fn from_data(data: &[u8], size: usize) -> Self {
        let mut objects = vec![];
        let mut data_to_process = data.clone();

        let repository = Repository::load(None).expect("should be a repository");

        while !data_to_process.is_empty() {
            let space = data_to_process.iter().position(|&x| x == 0x20).unwrap();
            let mode = &data_to_process[0..space];
            let mode = std::str::from_utf8(mode).expect("Invalid mode");
            data_to_process = &data_to_process[space + 1..];

            let null = data_to_process.iter().position(|&x| x == 0x00).unwrap();
            let name = &data_to_process[0..null];
            let name = std::str::from_utf8(name).expect("Invalid name");
            data_to_process = &data_to_process[null + 1..];

            let sha = &data_to_process[0..20];
            // binary decode the sha
            let sha = sha
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join("");
            data_to_process = &data_to_process[20..];

            let object = Object::load(&repository, &sha);

            objects.push(TreeEntry {
                mode: mode.to_string(),
                name: name.to_string(),
                sha,
                object,
            });
        }

        Tree {
            data: data.to_vec(),
            size,
            objects,
        }
    }
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

    pub fn cat(&self) -> String {
        let data = self.get_data();

        match self {
            Object::Blob(_) | Object::Commit(_) | Object::Tag(_) => match std::str::from_utf8(data)
            {
                Ok(s) => s.to_string(),
                Err(_) => format!("failed to decode content: {:?}", data),
            },
            Object::Tree(tree) => {
                // parse the tree object

                tree.objects
                    .iter()
                    .map(|entry| {
                        format!(
                            "{}\t{}\t{}\t{}\n",
                            entry.mode,
                            entry.object.get_type(),
                            entry.sha,
                            entry.name
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("")
            }
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

    pub fn load(repository: &Repository, sha: &str) -> Self {
        let path = repository.get_object_path(sha);
        let data = Self::read(&path);

        Self::deserialize(data)
    }

    /// deserialize an object from the repository by its uncompressed data
    fn deserialize(data: Vec<u8>) -> Self {
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
            "tree" => Object::Tree(Tree::from_data(data, size)),
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

    /// read an object file with a given path and decrompress it with zlib
    fn read(path: &str) -> Vec<u8> {
        let data = std::fs::read(path).expect("Failed to read object file");

        let mut z = ZlibDecoder::new(&data[..]);
        let mut s = Vec::new();
        //println!("{:?}", z.into_inner().read_to_string(buf));
        z.read_to_end(&mut s).expect("Failed to decompress object");

        s
    }

    /// serialize an object to a byte array
    fn serialize(&self) -> Vec<u8> {
        let t = match self {
            Object::Tag(_) => "tag",
            Object::Commit(_) => "commit",
            Object::Tree(_) => "tree",
            Object::Blob(_) => "blob",
        };

        let data = self.get_data();
        let size = data.len();

        let mut s = Vec::new();
        s.extend_from_slice(t.as_bytes());
        s.push(0x20);
        s.extend_from_slice(size.to_string().as_bytes());
        s.push(0x00);
        s.extend_from_slice(data);

        s
    }

    /// save an object to the repository
    pub fn save(&self, repository: &Repository) -> String {
        let data = self.serialize();
        let hash = self.hash();
        let path = repository.get_object_path(&hash);

        Self::write(&data, &path);
        hash
    }

    /// compress and byte array and write it to a file
    fn write(data: &Vec<u8>, path: &str) {
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(data).expect("Failed to compress object");
        let compressed = z.finish().expect("Failed to finish compression");

        // create the folder
        let folder = path
            .split("/")
            .take(path.split("/").count() - 1)
            .collect::<Vec<&str>>()
            .join("/");
        let _ = std::fs::create_dir_all(folder);

        std::fs::write(path, compressed).expect("Failed to write object file");
    }

    /// get the hash of an object
    pub fn hash(&self) -> String {
        let data = self.serialize();
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
