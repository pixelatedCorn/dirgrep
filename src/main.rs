use std::{
    fs::{self, DirEntry, ReadDir},
    path::{Path, PathBuf},
    vec,
};

use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'F', long)]
    fixed_string: bool,

    #[arg(short = 'D', long)]
    debug: bool,

    #[arg(short = 'j', long, default_value_t = false)]
    multi_thread: bool,

    pattern: String,
    dir: String,
}

struct Dir {
    iter: ReadDir,
}

impl Dir {
    fn from_path(path: &PathBuf) -> Option<Self> {
        let dir_iter = fs::read_dir(path);
        match dir_iter {
            Ok(iter) => Some(Self { iter }),
            Err(_) => None,
        }
    }
}

struct DirIter {
    dirs: Vec<Dir>,
    debug: bool,
}

impl DirIter {
    fn new(start: PathBuf, debug: bool) -> Self {
        let root = Dir::from_path(&start).unwrap();
        Self {
            dirs: vec![root],
            debug,
        }
    }

    fn handle_entry(&mut self, entry: DirEntry) -> Option<DirEntry> {
        let path = entry.path();
        let metadata = fs::metadata(path.as_path());
        match metadata {
            Ok(md) => {
                if md.is_symlink() {
                    return None;
                }

                if md.is_dir() {
                    self.dirs.push(Dir::from_path(&path).unwrap());
                    return None;
                }

                return Some(entry);
            }
            Err(er) => {
                if self.debug {
                    eprintln!("Error: {:?} | '{}'", er, path.display());
                }
                None
            }
        }
    }

    fn step_out(&mut self) {
        self.dirs.pop().expect("BUG: emptry stack (so)");
    }
}

impl Iterator for DirIter {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.dirs.is_empty() {
            let entry = self
                .dirs
                .last_mut()
                .expect("BUG: empty stack (gd)")
                .iter
                .next();
            match entry {
                Some(Ok(path)) => {
                    if let Some(val) = self.handle_entry(path) {
                        return Some(val);
                    }
                }
                Some(Err(_)) => return None,
                None => self.step_out(),
            };
        }
        None
    }
}

fn single_iter(args: Args) {
    let regex = if args.fixed_string {
        Regex::new(&regex::escape(&args.pattern)).expect("failed to parse pattern")
    } else {
        Regex::new(&args.pattern).expect("failed to parse pattern")
    };

    // no need to check for directory here, will be handled by iterator
    let path = Path::new(&args.dir);

    let iter = DirIter::new(path.to_path_buf(), args.debug);
    iter.for_each(|x| {
        let dir = x;
        let path = dir.path();
        if regex.is_match(path.to_str().unwrap()) {
            println!("{}", path.display());
        }
    });
}

fn main() {
    let args = Args::parse();

    single_iter(args.clone());
}
