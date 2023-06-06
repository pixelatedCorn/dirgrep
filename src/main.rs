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

    #[arg(short, long)]
    debug: bool,

    #[arg(short = 'j', long, default_value_t = false)]
    multi_thread: bool,

    pattern: String,
    dir: String,
}

struct Dir {
    parent: Option<Box<Dir>>,
    iter: ReadDir,
}

impl Dir {
    fn from_root(path: &PathBuf) -> Option<Self> {
        let dir_iter = fs::read_dir(path);
        match dir_iter {
            Ok(iter) => Some(Self {
                parent: None,
                iter,
            }),
            Err(_) => None,
        }
    }

    fn from_path(path: &PathBuf) -> Option<Self> {
        let dir_iter = fs::read_dir(path);
        match dir_iter {
            Ok(iter) => Some(Self {
                parent: None,
                iter,
            }),
            Err(_) => None,
        }
    }
}

struct DirIter {
    current: Box<Dir>,
    recurse: bool,
    debug: bool,
}

impl DirIter {
    fn new(start: PathBuf, recurse: bool, debug: bool) -> Self {
        let root = Dir::from_root(&start).unwrap();
        Self {
            current: Box::from(root),
            recurse,
            debug,
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
        let parent = std::mem::take(&mut self.current.parent);
        match parent {
            Some(parent) => {
                let dead_dir = std::mem::replace(&mut self.current, parent);
                // not really sure if this is necessary but just in case
                drop(dead_dir);
                self.next()
            },
            None => None,
        }
    }

    fn step_into_and_get_next(&mut self, path: PathBuf) -> Option<PathBuf> {
        let new_dir = Dir::from_path(&path);
        match new_dir {
            Some(dir) => {
                let parent = std::mem::replace(&mut self.current, Box::from(dir));
                self.current.parent = Some(parent);
            },
            None => {},
        };
        self.next()
    }
}

impl Iterator for DirIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.current.iter.next();
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

    let iter = DirIter::new(path.to_path_buf(), args.recursive, args.debug);
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
