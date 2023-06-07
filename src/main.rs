use std::{
    fs::{self, ReadDir, DirEntry},
    path::{Path, PathBuf},
};

use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'F', long)]
    fixed_string: bool,

    #[arg(short, long)]
    recursive: bool,

    #[arg(long)]
    debug: bool,

    #[arg(short = 'j', long, default_value_t = false)]
    multi_thread: bool,

    #[arg(short = 'd', long, default_value_t = 16)]
    max_depth: usize,

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
            Ok(iter) => Some(Self {
                iter,
            }),
            Err(_) => None,
        }
    }
}

struct DirIter {
    dirs: Vec<Dir>,
    recurse: bool,
    debug: bool,
    max_depth: usize,
    depth: usize,
}

impl DirIter {
    fn new(start: PathBuf, recurse: bool, debug: bool, max_depth: usize) -> Self {
        let root = Dir::from_path(&start).unwrap();
        let mut dirs = Vec::with_capacity(max_depth);
        dirs.push(root);
        Self {
            dirs,
            recurse,
            debug,
            max_depth,
            depth: 0,
        }
    }

    fn handle_entry(&mut self, entry: DirEntry) -> Option<PathBuf> {
        let path = entry.path();
        let metadata = fs::metadata(path.as_path());
        match metadata {
            Ok(metadata) => {
                let ft = metadata.file_type();
                if ft.is_symlink() {
                    self.next()
                } else if ft.is_dir() {
                    if !self.recurse {
                        return self.next();
                    }
                    self.step_into_and_get_next(path)
                } else {
                    Some(path)
                }
            },
            Err(e) => {
                if self.debug { 
                    eprintln!("Error at path '{}': {:?}", path.display(), e);
                }
                self.next()
            }
        }
    }

    fn step_up_and_get_next(&mut self) -> Option<PathBuf> {
        self.dirs.pop();
        if self.depth > 0 {
            self.depth -= 1;
        }
        self.next()
    }

    fn step_into_and_get_next(&mut self, path: PathBuf) -> Option<PathBuf> {
        if self.depth + 1 < self.max_depth {
            let new_dir = Dir::from_path(&path);
            match new_dir {
                Some(dir) => {
                    self.dirs.push(dir);
                    self.depth += 1;
                },
                None => {},
            };
        }
        self.next()
    }
}

impl Iterator for DirIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.dirs.len() == 0 {
            return None;
        }
        let entry = self.dirs[self.depth].iter.next();
        match entry {
            Some(val) => {
                match val {
                    Ok(path) => self.handle_entry(path),
                    Err(_) => self.next(),
                }
            },
            None => {
                self.step_up_and_get_next()
            }
        }
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

    let iter = DirIter::new(path.to_path_buf(), args.recursive, args.debug, args.max_depth);
    iter.for_each(|x| {
        let string = x.to_str().unwrap();
        if regex.is_match(string) {
            println!("{}", string);
        }
    });
}

fn main() {
    let args = Args::parse();

    single_iter(args);
}
