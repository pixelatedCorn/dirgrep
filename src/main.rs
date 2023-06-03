use std::{path::Path, io, fs};

use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'F', long)]
    fixed_string: bool,

    #[arg(short, long)]
    recursive: bool,

    pattern: String,
    dir: String,
}

fn process_dir(path: &Path, args: &Args, regex: &Regex) -> io::Result<(usize, usize)> {
    let mut count = 0;
    let mut total = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if args.recursive {
                let (t, c) = process_dir(&path, &args, &regex)?;
                count += c;
                total += t;
            }
        } else {
            total += 1;
            if regex.is_match(&path.file_name().unwrap().to_str().unwrap()) {
                println!("{}", path.display());
                count += 1;
            };
        }
    }

    Ok((total, count))
}

fn main() {
    let args = Args::parse();
    let regex = if args.fixed_string {
        Regex::new(&regex::escape(&args.pattern)).expect("failed to parse pattern")
    } else {
        Regex::new(&args.pattern).expect("failed to parse pattern")
    };

    let path = Path::new(&args.dir);

    if !path.is_dir() {
        panic!("path is not directory");
    }

    let (total, count) = process_dir(&path, &args, &regex).expect("fatal error");
    println!("{} files checked...", total);
    println!("{} matches found!", count);
}
