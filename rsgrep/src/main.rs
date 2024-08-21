use std::{env, fs, path::{Path, PathBuf}, usize};
mod rscli;
use rscli::{Rscli, Flag};
use walkdir::{WalkDir, DirEntry};
use ansi_term::Colour;
use regex::Regex;


const REGEX_FLAG: &str = "R";
const REGEX_DESC: &str = "Match with regex.";
const RECURSIVE_FLAG: &str = "r";
const RECURSIVE_DESC: &str = "Search recursively.";
const LINE_NUMBER_FLAG: &str = "n";
const LINE_NUMBER_DESC: &str = "Print line number.";
const NO_MATCH_ONLY_FLAG: &str = "L";
const NO_MATCH_ONLY_DESC: &str = "Print file names without match.";
const MATCH_ONLY_FLAG: &str = "l";
const MATCH_ONLY_DESC: &str = "Print file names with match.";
const FOLLOW_SYMLINK_FLAG: &str = "s";
const FOLLOW_SYMLINK_DESC: &str = "Follow symbolic link.";
const EXCLUDE_FLAG: &str = "e";
const EXCLUDE_DESC: &str = "Exclude files/directories.";
const PATH_FLAG: &str = "p";
const PATH_DESC: &str = "Specify paths to search into.";

struct LineMatch {
    line: String,
    line_number: usize,
    start: usize,
    end: usize
}

struct Match {
    path: PathBuf,
    lines: Vec<LineMatch>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let regex = Flag {
        id: REGEX_FLAG,
        description: REGEX_DESC,
        values: vec![],
    };

    let recursive = Flag {
        id: RECURSIVE_FLAG,
        description: RECURSIVE_DESC,
        values: vec![],
    };
    
    let line_number = Flag {
        id: LINE_NUMBER_FLAG,
        description: LINE_NUMBER_DESC,
        values: vec![],
    };

    let no_match_only = Flag {
        id: NO_MATCH_ONLY_FLAG,
        description: NO_MATCH_ONLY_DESC,
        values: vec![],
    };

    let match_only = Flag {
        id: MATCH_ONLY_FLAG,
        description: MATCH_ONLY_DESC,
        values: vec![],
    };

    let follow_symklink = Flag {
        id: FOLLOW_SYMLINK_FLAG,
        description: FOLLOW_SYMLINK_DESC,
        values: vec![],
    };

    let exclude = Flag {
        id: EXCLUDE_FLAG,
        description: EXCLUDE_DESC,
        values: vec![],
    };

    let paths = Flag {
        id: PATH_FLAG,
        description: PATH_DESC,
        values: vec![],
    };

    let program_name = args.get(0).expect("[ERROR] Program name should exist.").clone();
    let program_flags = vec![regex, recursive, line_number, no_match_only, match_only, follow_symklink, exclude, paths];
    let examples = vec![
        format!("{program_name} 'foreach' -rn  // Print each line and number containing 'foreach'"), 
        format!("{program_name} 'foreach' -rL  // Print each file that does not contain 'foreach'"), 
        format!("{program_name} 'foreach' 'another pattern' -p /home -rL  // Print each file that does not contain 'foreach'"), 
        ];
    let mut cli = Rscli::new(program_name, program_flags, examples);
    cli.parse_args(&args);

    if !validate(&cli) {
        cli.usage();
    }

    search(&cli);

}

fn validate(cli: &Rscli) -> bool {
    if cli.matched_flags.len() == 0 {
        return false;
    }

    if cli.free_args.len() == 0 && cli.find_user_flag_by_id(REGEX_FLAG).is_none(){
        return false;
    }

    return true;
}



fn filter(entry: &DirEntry, cli: &Rscli) -> bool {
    
    let recursive = cli.find_user_flag_by_id(RECURSIVE_FLAG);
    let exclude = cli.find_user_flag_by_id(EXCLUDE_FLAG);

    if let Ok(metadata)  = entry.metadata() {
        if metadata.is_dir() && recursive.is_none() {
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
    let follow_symlink = cli.find_user_flag_by_id(FOLLOW_SYMLINK_FLAG).is_some();

    for path in paths {
        search_in_path(&path, cli, follow_symlink);
    }
}

fn get_search_paths(cli: &Rscli) -> Vec<PathBuf> {
    if let Some(path_flag) = cli.find_user_flag_by_id(PATH_FLAG) {
        path_flag.values.iter().map(PathBuf::from).collect()
    } else {
        vec![PathBuf::from(".")]
    }
}

fn search_in_path(path: &Path, cli: &Rscli, follow_symlink: bool) {
    let walker = WalkDir::new(path)
        .follow_links(follow_symlink)
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
            if absolute.is_file() {
                match_file(&absolute, cli);
            }
        }
        Err(_) => return,
    }
}

fn match_file(file_path: &Path, cli: &Rscli) {
    let regex = cli.find_user_flag_by_id(REGEX_FLAG);
    let patterns = &cli.free_args;

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let mut line_matches: Vec<LineMatch> = vec![];
            for (idx, line) in content.lines().enumerate() {
                if let Some(regex) = regex {
                    if let Some(match_line) = match_regex(line, idx, &regex.values) {
                        line_matches.push(match_line);
                    }
                } else {
                    if let Some(match_line) = match_pattern(line,idx, patterns) {
                        line_matches.push(match_line);
                    }
                }
            }
            print_fmt(Match {
                path: file_path.to_path_buf(),
                lines: line_matches,
            }, cli);
        }
        Err(_) => return,
    }
}

fn match_regex(line: &str, line_number: usize, regex: &Vec<String>) -> Option<LineMatch> {
    for reg in regex {
        if let Ok(re) = Regex::new(reg){
            if let Some(res) = re.find(line) {
                return Some(LineMatch {
                    line: line.to_string(),
                    line_number: line_number,
                    start: res.start(),
                    end: res.end()
                })
            }
        } else {
            eprintln!("[ERROR] Failed to compile regex: {reg}");
        }
    }
    return None;
}

fn match_pattern(line: &str, line_number: usize, patterns: &Vec<String>) -> Option<LineMatch> {
    for pattern in patterns {
        if let Some(start) = line.find(pattern) {
            return Some(LineMatch {
                line: line.to_string(),
                line_number: line_number,
                start: start,
                end: start + pattern.len()
            })
        }
    }

    return None;
}

fn print_fmt(matching: Match, cli: &Rscli) {
    let line_number = cli.find_user_flag_by_id(LINE_NUMBER_FLAG);
    let matching_files = cli.find_user_flag_by_id(MATCH_ONLY_FLAG);
    let no_matching_files = cli.find_user_flag_by_id(NO_MATCH_ONLY_FLAG);

    
    if no_matching_files.is_some() && matching.lines.len() == 0 {
        print_path(&matching.path);
        println!();
        return;
    } else if matching_files.is_some() && matching.lines.len() > 0 {
        print_path(&matching.path);
        println!();
        return;
    } else if line_number.is_some() {
        for line in matching.lines {
            print_path(&matching.path);
            print_column();
            print_line_number(line.line_number + 1);
            print_column();
            print_line(line);
            println!();
        }
    } else if no_matching_files.is_none() && matching_files.is_none(){
        for line in matching.lines {
            print_path(&matching.path);
            print_column();
            print_line(line);
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

fn print_column() {
    let style = Colour::Cyan.bold();
    print!("{}", style.paint(":"));
}

fn print_line_number(line_number: usize) {
    let style = Colour::Green.bold();
    print!("{}", style.paint(line_number.to_string()));
}

fn print_line(line: LineMatch) {
    let style = Colour::Red.bold();
    print!("{}", &line.line[0..line.start]);
    print!("{}", style.paint(&line.line[line.start..line.end]));
    print!("{}", &line.line[line.end..]);
}