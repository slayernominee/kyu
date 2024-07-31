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

        folder: Option<String>,
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
    ShowRef {
        reference: Option<String>,
    },
    Status,
    Tag,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Init { path } => init(path),
        Commands::CatFile {
            object_type: _,
            hash,
        } => cat_file(&hash),
        Commands::HashObject { path, write, type_ } => hash_object(path, write, type_),
        Commands::Log { commit } => log(commit),
        Commands::LsTree { hash } => cat_file(&hash),
        Commands::Checkout { commit, folder } => checkout(commit, folder),
        Commands::ShowRef { reference } => show_ref(reference),
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

fn cat_file(hash: &str) {
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
    let hash = rep.ref_resolve(&commit);

    match hash {
        Ok(hash) => {
            let commit = Object::load(&rep, &hash);
            let commit = match commit {
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

fn show_ref(reference: Option<String>) {
    let rep = Repository::load(None).unwrap();

    if reference.is_none() {
        let refs = rep.get_refs();
        for (refname, hash) in refs {
            println!("{}\t{}", hash, refname);
        }
        return;
    }
    let reference = reference.unwrap();

    let r = rep.ref_resolve(&reference);

    let r = match r {
        Ok(r) => r,
        Err(e) => match e {
            _ => format!("fatal: {:?}", e),
        },
    };

    println!("{}\t{}", r, reference);
}

fn checkout(commit_or_ref: String, folder: Option<String>) {
    let pwd = std::env::current_dir().unwrap();
    let path_to_checkout = pwd.to_string_lossy();
    let mut path_to_checkout =
        path_to_checkout.to_string() + "/" + &folder.unwrap_or("".to_string());

    if std::path::Path::new(&path_to_checkout).is_dir() && !path_to_checkout.ends_with("/") {
        path_to_checkout.push_str("/");
    }

    // if files_or_folders is None, checkout the whole commit

    let rep = Repository::load(None).unwrap();
    let commit = rep
        .ref_resolve(&commit_or_ref)
        .expect("Invalid Reference / Commit");
    let commit = Object::load(&rep, &commit);

    let tree = match commit {
        Object::Commit(c) => {
            let t = Object::load(&rep, &c.get_tree());
            match t {
                Object::Tree(t) => t,
                _ => panic!("head should be a commit object"),
            }
        }
        Object::Tree(t) => t,
        _ => panic!("head should be a commit object"),
    };

    tree.checkout(rep.get_workdir().to_owned(), path_to_checkout.to_string());
}
