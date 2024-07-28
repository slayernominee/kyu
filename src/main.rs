use clap::{Parser, Subcommand};
use repository::Repository;

mod objects;
mod repository;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        files: Vec<String>,
    },
    CatFile,
    Commit {
        #[arg(short, long)]
        message: Option<String>,
    },
    CheckIgnore,
    Checkout,
    /// Convert an file into a blob object
    HashObject {
        path: String,
    },
    /// Initialize a new git repository
    Init {
        path: Option<String>,
    },
    Log,
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
        Commands::HashObject { path } => hash_object(path),
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

fn hash_object(path: String) {
    let rep = Repository::load(None).unwrap();
    let obj = objects::Object::deserialize(&rep, "1e143c83828e1f647f8597c25e12bd6c24cb0979");
    println!("{:?}", obj.get_type());
}
