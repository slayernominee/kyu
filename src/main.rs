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
    Checkout,
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
    LsTree,
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
