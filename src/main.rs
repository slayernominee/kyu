use clap::{Parser, Subcommand};

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
    HashObject,
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

    println!("rep: {:?}", rep);
}
