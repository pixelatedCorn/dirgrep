use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
    },
    time::Instant,
};

use clap::Parser;
use regex::Regex;

use rayon::{Scope};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'F', long)]
    fixed_string: bool,

    #[arg(short, long)]
    recursive: bool,

    #[arg(short = 'j', long, default_value_t = false)]
    multi_thread: bool,

    pattern: String,
    dir: String,
}

fn process_dir_sp(path: &Path, regex: &Regex, recurse: bool) -> io::Result<(usize, usize)> {
    let mut count = 0;
    let mut total = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if recurse {
                let (t, c) = process_dir_sp(&path, &regex, recurse)?;
                count += c;
                total += t;
            }
        } else {
            total += 1;
            if regex.is_match(&path.file_name().unwrap().to_str().unwrap()) {
                //println!("{}", path.display());
                count += 1;
            };
        }
    }

    Ok((total, count))
}

fn process_dir_mt_sp(path: &Path, regex: Regex) {
    let files = Arc::from(Mutex::from(vec![path.to_path_buf()]));
    let regex = Arc::from(regex);
    fn process_dir(
        scope: &Scope,
        path: PathBuf,
        files: Arc<Mutex<Vec<PathBuf>>>,
        regex: Arc<Regex>,
    ) {
        let entries = fs::read_dir(path).unwrap();
        let mut dir_files = vec![];

        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                let arc = files.clone();
                let regex = regex.clone();
                scope.spawn(|s| process_dir(s, path, arc, regex));
            } else {
                if regex.is_match(path.to_str().unwrap()) {
                    dir_files.push(path);
                }
            }
        }

        let mut files = files.lock().unwrap();

        files.append(&mut dir_files);
    }

    let files_clone = files.clone();
    rayon::scope(|scope| {
        process_dir(scope, path.to_path_buf(), files_clone, regex);
    });

    let mutex = Arc::try_unwrap(files).unwrap();
    let _files = mutex.into_inner().unwrap();

    _files.iter().for_each(|x| {
        x.to_str().unwrap();
    });
    //println!("{} files checked...", files.len());
    //println!("{} matches found!", files.len());
}

struct DirIter {
    paths: Vec<PathBuf>
}

impl DirIter {
    fn new(start: PathBuf) -> Self {
        Self {
            paths: vec![start],
        }
    }

    fn process_path(&mut self, path: PathBuf) -> PathBuf {
        if path.is_dir() {
            let paths = fs::read_dir(path).unwrap();
            self.paths.append(&mut paths.map(|x| {x.unwrap().path()}).collect::<Vec<PathBuf>>());
            let last = self.paths.pop().unwrap();
            return self.process_path(last)
        } else {
            return path;
        }
    }
}

impl Iterator for DirIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.paths.pop();
        match cur {
            Some(path) => {
                Some(self.process_path(path))
            },
            None => None,
        }
    }
}

fn single_iter(path: PathBuf, regex: &Regex) {
    let iter = DirIter::new(path);
    iter.for_each(|x| {
        let string = x.to_str().unwrap();
        if regex.is_match(string) {
            println!("{}", string);
        }
    });
}

fn temp_main() {
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

    single_iter(path.to_path_buf(), &regex);
}

fn main() {
    temp_main();
    return;
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

    const NUM_ITERS: usize = 10;
    //println!("running single pass...");
    let mut before = Instant::now();
    //for _ in 0..NUM_ITERS {
        //process_dir_sp(path, &regex, args.recursive).unwrap();
    //}
    //let single = before.elapsed() / NUM_ITERS as u32;

    //println!("running multi thread single pass...");
    //before = Instant::now();
    //for _ in 0..NUM_ITERS {
        //process_dir_mt_sp(path, regex.clone());
    //}
    //let multi_thread_sp = before.elapsed() / NUM_ITERS as u32;

    println!("running single iter...");
    //before = Instant::now();
    for _ in 0..NUM_ITERS {
        single_iter(path.to_path_buf(), &regex);
    }
    let single_iter = before.elapsed() / NUM_ITERS as u32;
    //println!("single pass: {:.2?}", single);
    println!("single iter: {:.2?}", single_iter);
    //println!("multi thread single pass: {:.2?}", multi_thread_sp);
}
