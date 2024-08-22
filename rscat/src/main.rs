mod rscli;

use std::env;
use std::fs::read_to_string;
use rscli::Rscli;

fn validate(drcli: &Rscli) -> bool {
    if drcli.free_args.len() == 0 {
        return false;
    }

    return true;
}

fn read_file(file_path: &String) -> Option<String> {
    match read_to_string(file_path) {
        Ok(content) => {
            Some(content)
        }, 
        Err(err) => {
            eprintln!("{err}");
            None
        }
    }
}

fn start(cli: &Rscli) {
    for file in cli.free_args.iter() {
        if let Some(content) = read_file(file) {
            println!("{content}");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let program_name = args.get(0).expect("[ERROR] Program name should exist.").clone();
    let program_flags = vec![];
    let examples = vec![
        format!("{program_name} <file1.txt> // Print in console file1.txt content"),
        format!("{program_name}  <file1.txt> <file2.txt> > out.txt // Concatenate two files"),
        ];
    let mut cli = Rscli::new(program_name, program_flags, examples);
    cli.parse_args(&args);

    if !validate(&cli) {
        cli.usage();
    }

    start(&cli);
}
