mod rscli;

use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use ansi_term::Colour;
use rscli::{Flag, Rscli};
use walkdir::{WalkDir, DirEntry};

const DIR_FLAG: &str = "d";
const DIR_DESC: &str = "Specify directory to search into";
const RECURSIVE_FLAG: &str = "r";
const RECURSIVE_DESC: &str = "Find files recursively";
const EXCLUDE_FLAG: &str = "e";
const EXCLUDE_DESC: &str = "Exclude dir or files";
const FOLLOW_SYMLINK_FLAG: &str = "s";
const FOLLOW_SYMLINK_DESC: &str = "Follow symlink";

fn validate(drcli: &Rscli) -> bool {
    if drcli.free_args.len() == 0 {
        return false;
    }

    return true;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    
    let dir = Flag {
        id: DIR_FLAG, 
        description: DIR_DESC,
        values: vec![]
    };
    let recursive = Flag {
        id: RECURSIVE_FLAG, 
        description: RECURSIVE_DESC,
        values: vec![]
    };
    let exclude = Flag {
        id: EXCLUDE_FLAG, 
        description: EXCLUDE_DESC,
        values: vec![]
    };
    let symlink = Flag {
        id: FOLLOW_SYMLINK_FLAG, 
        description: FOLLOW_SYMLINK_DESC,
        values: vec![]
    };

    let program_name = args.get(0).expect("[ERROR] Program name should exist.").clone();
    let program_flags = vec![dir, recursive, exclude, symlink];
    let examples = vec![
        format!("{program_name} \"pattern\" -d <dir>"),
        format!("{program_name} \"pattern\" -r"),
        ];
    let mut cli = Rscli::new(program_name, program_flags, examples);
    cli.parse_args(&args);

    if !validate(&cli) {
        cli.usage();
    }

    search(&cli);
}



fn filter(entry: &DirEntry, cli: &Rscli) -> bool {
    
    let recursive = cli.find_user_flag_by_id(RECURSIVE_FLAG);
    let exclude = cli.find_user_flag_by_id(EXCLUDE_FLAG);

    if let Ok(metadata)  = entry.metadata() {
        if metadata.is_dir() && recursive.is_none() && !entry.file_name().eq_ignore_ascii_case(".") {
            return false;
        }
        if let Some(exclude) = exclude {
            if let Ok(file_name) =  fs::canonicalize(entry.path()) {
                for e in exclude.values.clone() {
                    if file_name.to_str().unwrap().contains(&e) {
                        return false;
                    }
                }
            }
        }
    }
    return true;
}

fn search(cli: &Rscli) {
    let paths = get_search_paths(cli);

    for path in paths {
        search_in_path(&path, cli);
    }
}

fn get_search_paths(cli: &Rscli) -> Vec<PathBuf> {
    if let Some(path_flag) = cli.find_user_flag_by_id(DIR_FLAG) {
        path_flag.values.iter().map(PathBuf::from).collect()
    } else {
        vec![PathBuf::from(".")]
    }
}

fn search_in_path(path: &Path, cli: &Rscli) {
    let walker = WalkDir::new(path)
        .follow_links(cli.find_user_flag_by_id(FOLLOW_SYMLINK_FLAG).is_some())
        .into_iter()
        .filter_entry(|e| filter(e, cli));

    for entry in walker {
        if let Ok(e) = entry {
            process_entry(&e, cli);
        }
    }
}

fn process_entry(entry: &DirEntry, cli: &Rscli) {
    match entry.path().canonicalize() {
        Ok(absolute) => {       
            match_file(&absolute, cli);
        }
        Err(_) => return,
    }
}

fn match_file(path: &PathBuf, cli: &Rscli) {
    for pattern in cli.free_args.iter() {
        if path.to_str().unwrap().contains(pattern) {
            print_path(path);
            println!();
        }
    }
}

fn print_path(path: &PathBuf) {
    let mut path_str = path.to_string_lossy().to_string();
    if path_str.starts_with(r"\\?\") {
        path_str = path_str[4..].to_string();
    }
    
    let style = Colour::Purple.bold();
    print!("{}", style.paint(path_str));
}