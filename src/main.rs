use clap::{Parser, Subcommand, ValueEnum};
use objects::{Blob, Object, KVLM};
use repository::{RepError, Repository};

mod logscreen;
mod objects;
mod repository;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Debug, Clone, PartialEq, Eq)]
enum ObjectType {
    Blob,
    Commit,
    Tree,
    Tag,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        files: Vec<String>,
    },
    /// Print the contents of a blob object
    CatFile {
        #[arg(value_enum)]
        object_type: ObjectType,

        hash: String,
    },
    Commit {
        #[arg(short, long)]
        message: Option<String>,
    },
    CheckIgnore,
    /// Set the current branch / specified folders / files to a specific commit / tree
    Checkout {
        commit: String,

        files_or_folders: Option<String>,
    },
    /// Convert an file into a blob object
    HashObject {
        path: String,

        #[arg(short, long)]
        write: bool,

        #[arg(long, value_enum, name = "type")]
        type_: Option<ObjectType>,
    },
    /// Initialize a new git repository
    Init {
        path: Option<String>,
    },
    /// Show the commit history
    Log {
        #[arg(default_value = "HEAD")]
        commit: String,
    },
    LsFiles,
    LsTree {
        hash: String,
    },
    RevParse,
    Rm {
        files: Vec<String>,
    },
    ShowRef,
    Status,
    Tag,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Init { path } => init(path),
        Commands::CatFile { object_type, hash } => cat_file(object_type, &hash),
        Commands::HashObject { path, write, type_ } => hash_object(path, write, type_),
        Commands::Log { commit } => log(commit),
        Commands::LsTree { hash } => cat_file(ObjectType::Tree, &hash),
        Commands::Checkout {
            commit,
            files_or_folders,
        } => checkout(commit, files_or_folders),
        _ => println!("Not implemented yet"),
    }
}

fn init(path: Option<String>) {
    let rep = repository::Repository::init(path);

    if rep.is_err() {
        println!("Failed to initialize repository: {:?}", rep.err().unwrap());
        return;
    }

    let rep = rep.ok().unwrap();

    println!(
        "Initialized new repository in the directory: {}",
        rep.get_workdir()
    );
}

fn cat_file(object_type: ObjectType, hash: &str) {
    let rep = Repository::load(None).unwrap();
    // TODO: Implement loading with short hashes etc.
    let obj = Object::load(&rep, hash);

    println!("{}", obj.cat());
}

fn hash_object(path: String, write: bool, object_type: Option<ObjectType>) {
    if object_type.is_some() && object_type.unwrap() != ObjectType::Blob {
        unimplemented!("Only blobs can be hashed for now")
    }

    let rep = Repository::load(None).unwrap();
    let obj = Blob::from_file(&path);

    if write {
        obj.save(&rep);
    }

    println!("{}", obj.hash());
}

fn log(commit: String) {
    let rep = Repository::load(None).unwrap();
    let hash = rep.get_last_commit_hash();

    match hash {
        Ok(hash) => {
            let commit = Object::load(&rep, &hash);
            let mut commit = match commit {
                Object::Commit(c) => c,
                _ => panic!("head should be a commit object"),
            };

            logscreen::display_log(commit, rep);
        }
        Err(RepError::NoCommitsInBranch(branch)) => {
            println!(
                "fatal: your current branch '{}' does not have any commits yet",
                branch
            );
        }
        Err(e) => {
            println!("fatal: {:?}", e);
        }
    }
}

fn checkout(commit: String, files_or_folders: Option<String>) {
    // if files_or_folders is None, checkout the whole commit

    let rep = Repository::load(None).unwrap();
    let commit = Object::load(&rep, &commit);

    let pwd = std::env::current_dir().unwrap();

    let checkout_point = match files_or_folders {
        Some(f) => {
            // check if its a file or folder, if its a folder set the checkout to folder/

            let path = pwd.join(f.clone());
            if path.is_dir() {
                if f.ends_with("/") {
                    format!("/{}", f)
                } else {
                    format!("/{}/", f)
                }
            } else {
                format!("/{}", f)
            }
        }
        None => "/".to_string(),
    };

    let tree = match commit {
        Object::Commit(c) => {
            let obj = Object::load(&rep, c.get_tree().as_str());

            match obj {
                Object::Tree(t) => t,
                _ => panic!("commit is not a valid commit / tree hash"),
            }
        }
        Object::Tree(t) => t,
        _ => panic!("commit is not a valid commit / tree hash"),
    };

    // navigate to the tree to the checkout point

    let checkout_file = checkout_point.ends_with("/"); // wheter to checkout a file or a folder

    // while checkout point has more than one / we need to navigate to the correct tree
    let mut tree = tree;
    let mut path = checkout_point.clone();

    while path.matches("/").count() > 1 {
        let mut path_parts = path.split("/").collect::<Vec<&str>>();
        path_parts.remove(0); // remove the first empty string
        let path_part = path_parts.remove(0);

        let mut found = false;
        for entry in tree.get_objects() {
            if entry.get_name() == path_part {
                let obj = Object::load(&rep, entry.get_hash());
                match obj {
                    Object::Tree(t) => {
                        tree = t;
                        found = true;
                        break;
                    }
                    _ => {
                        println!("fatal: path '{}' is not a folder", path_part);
                        return;
                    }
                }
            }
        }

        if !found {
            println!("fatal: path '{}' not found in the tree", path_part);
            return;
        }

        path = format!("/{}", path_parts.join("/"));
    }

    println!("{}", tree.display_objects());
}
