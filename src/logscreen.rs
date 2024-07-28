use colored::*;
use std::collections::VecDeque;

use crate::objects::{Commit, Object, KVLM};
use crate::repository::Repository;

pub fn display_log(mut commit: Commit, rep: Repository) {
    let mut commits_to_visit: VecDeque<Commit> = VecDeque::new();
    commits_to_visit.push_back(commit);

    while !commits_to_visit.is_empty() {
        let commit = commits_to_visit.pop_front().unwrap();
        let commit = Object::load(&rep, &commit.hash());
        let commit = match commit {
            Object::Commit(c) => c,
            _ => panic!("head should be a commit object"),
        };

        println!("{} {}", "commit".cyan(), commit.hash().blue());

        if commit.get_parents().len() > 1 {
            println!(
                "Merge: {} -> {}",
                commit.get_parents()[0],
                commit.get_parents()[1]
            );
        }

        println!("Author: {}", commit.get_author());
        //println!("Date: {}", commit.get_date());
        println!();
        println!("    {}", commit.get_message());
        println!();

        for parent in commit.get_parents() {
            let parent = Object::load(&rep, &parent);
            let parent = match parent {
                Object::Commit(c) => c,
                _ => panic!("head should be a commit object"),
            };

            commits_to_visit.push_back(parent);
        }
    }
}
