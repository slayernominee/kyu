use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Utc};
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

        let author_data = commit.get_author();
        let author_data = author_data.split(" ").collect::<Vec<&str>>();
        let author = author_data[0..2].join(" ");
        println!("Author: {}", author);

        if author_data.len() > 3 {
            let date = author_data[2].parse::<i64>();
            let offset = author_data[3].parse::<i32>().unwrap_or_else(|_| 0);
            let offset = FixedOffset::east_opt(offset * 36).unwrap(); // / 100 * 3600 = * 36

            match date {
                Ok(date) => {
                    let date = Utc.timestamp_opt(date, 0).unwrap();
                    let date = date.with_timezone(&offset);
                    println!(
                        "Date: {}, {} {} {} {} {}",
                        date.weekday(),
                        date.day(),
                        date.format("%b"),
                        date.year(),
                        date.time(),
                        date.timezone(),
                    );
                }
                Err(_) => {
                    println!("Date: unknown");
                }
            }
        }

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
