use std::process::exit;

#[derive(Debug, Clone)]
pub struct Flag {
    pub id: &'static str,
    pub description: &'static str,
    pub values: Vec<String>,
}

pub struct Rscli {
    pub program_name: String,
    pub program_flags: Vec<Flag>,
    pub matched_flags: Vec<Flag>,
    pub free_args: Vec<String>,
    pub examples: Vec<String>
}

impl Rscli {
    pub fn new(program_name: String, program_flags: Vec<Flag>, examples: Vec<String>) -> Self {
        Rscli {
            program_name,
            program_flags,
            matched_flags: vec![],
            free_args: vec![],
            examples
        }
    }

    pub fn parse_args(&mut self, args: &[String]) {
        for arg in &args[1..] {
            if arg.starts_with('-') {
                self.parse_flags(arg);
            } else {
                self.handle_non_flag(arg);
            }
        }
    }

    pub fn parse_flags(&mut self, arg: &str) {
        for c in arg.chars().skip(1) { // Skip the leading '-'
            if let Some(flag) = self.find_program_flag_by_id(&c.to_string()) {
                self.matched_flags.push(flag.clone());
            } else {
                println!("[ERROR] Unknown flag: {c}");
                self.usage();
            }
        }
    }

    pub fn handle_non_flag(&mut self, arg: &str) {
        if let Some(flag) = self.matched_flags.last_mut() {
            flag.values.push(arg.to_owned());
        } else {
            self.free_args.push(arg.to_owned());
        }
    }
    

    pub fn find_program_flag_by_id(&self, id: &str) -> Option<&Flag> {
        self.program_flags.iter().find(|f| f.id == id)
    }
    
    pub fn find_user_flag_by_id(&self, id: &str) -> Option<&Flag> {
        self.matched_flags.iter().find(|f| f.id == id)
    }

    pub fn usage(&self) {
        println!();
        println!("USAGE: {} [VALUES] [OPTIONS] [ARGS]", self.program_name);
        println!("OPTIONS:");
        for flag in &self.program_flags {
            println!("    -{}    {}", flag.id, flag.description);
        }

        println!("EXAMPLES:");
        for example in &self.examples {
            println!("{example}");
        }
        println!();
        exit(1);
    }
}

