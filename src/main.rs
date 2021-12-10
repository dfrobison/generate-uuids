use chrono::prelude::*;
use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use uuid::Uuid;

fn display_usage() {
    println!("Usage: generate_uuids number_of_uuids [hyrax | bagheera | hornet | bumblebee | coati]");
}

fn main() {
    let mut new_uuids:Vec<String> = Vec::new();
    let mut from_files_uuids:Vec<String> = Vec::new();
    let today = Utc::today().format("%Y_%m_%d");
    let args: Vec<String> = env::args().collect();

    if args.iter().count() != 3 {
        display_usage();
        process::exit(1);
    }

    let number_of_uuids_to_generate: i32 = args[1].parse().expect("Need number_of_uuids");
    let camera_type = &args[2];

    match camera_type.as_str() {
        "hyrax" | 
        "bagheera" | 
        "hornet" | 
        "bumblebee" | 
        "coati" => {println!("Generating {} new UUIDs for {}", number_of_uuids_to_generate, camera_type);}
        _ => {display_usage(); process::exit(1);}
    }


    // Generate initial UUIDs
    for _ in 1..=number_of_uuids_to_generate {
        new_uuids.push(Uuid::new_v4().to_hyphenated().to_string());
    }

    let before_dedup_count = new_uuids.iter().count();

    // Make sure that duplicate UUIDs have been generated
    new_uuids.sort();
    new_uuids.dedup();

    if new_uuids.iter().count() != before_dedup_count {
        println!("Failed to generate enough unique UUIDs");
        process::exit(1);
    }

    // Create a vector of all the existing UUIDs that have been created.
    let re = Regex::new(r"(?:hornet|bagheera|hyrax|bumblebee|coati).*txt").unwrap();
    let files = fs::read_dir("camera-uuids").unwrap();

    files
        .filter_map(Result::ok)
        .filter_map(|d| {
            d.path()
                .to_str()
                .and_then(|f| if re.is_match(f) { Some(d) } else { None })
        })
        .for_each(|d| {
            println!("Reading {:?}", d);

            if let Ok(lines) = read_lines(d.path()) {
                for line in lines {
                    if let Ok(ip) = line {
                        from_files_uuids.push(ip)
                    }
                }
            }
        });

    // Create a combined vector with all the UUIDs and validate there are no duplicates
    let mut combined: Vec<&String> = new_uuids.iter().chain(from_files_uuids.iter()).collect();
    let total_combined = combined.iter().count();

    combined.sort();
    combined.dedup();

    if total_combined != combined.iter().count() {
        println!("UUID collisions were detectd");
        process::exit(1);
    }

    let new_file = camera_type.to_owned() + "_" + &today.to_string() + ".txt";

    // Write out the new UUIDs
    let mut file = std::fs::File::create(new_file).expect("create failed");
    for i in &new_uuids {
        write!(file, "{}\n", i).expect("failed writing");
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
