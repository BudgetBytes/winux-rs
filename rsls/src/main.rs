use std::fs::{self, Metadata, DirEntry};
use ansi_term::Colour;

const SECONDS_IN_MINUTE: u64 = 60;
const SECONDS_IN_HOUR: u64 = 3600;
const SECONDS_IN_DAY: u64 = 86400;
const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const CEST_UTC_ITALY: u64 = 2;
const _CET_UTC_ITALY: u64 = 1;

fn main() {
    let path: String;
    if let Some(arg) = std::env::args().nth(1) {
        path = arg;
    } else {
        path = ".".to_string();
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if let Some(filename) = entry.file_name().to_str() { 
                        print_type(&metadata);
                        print_permissions(&metadata);
                        // print_size(&metadata);
                        print_last_modified(&metadata);
                        print_filename(&metadata, &entry, filename);
                        println!();
                    }
                } else {
                    println!("Couldn't get metadata for {:?}", entry.path());
                }
            }
        }
    }
}

fn print_type(metadata: &Metadata) {
    
    if metadata.is_dir() {
        let style = Colour::Blue.bold();
        print!("{}", style.paint("d")); 
    } else if metadata.is_file() {
        print!("{}", "-");
    } else {
        let style = Colour::Cyan.bold();
        print!("{}", style.paint("l")); 
    }
}

fn print_permissions(metadata: &Metadata)  {
    let yellow = Colour::Yellow.bold();
    let red = Colour::Red.bold();
    
    if metadata.permissions().readonly() {
        print!("{}{}", yellow.paint("r"), red.paint("-"));
    } else {
        print!("{}{}", yellow.paint("r"), red.paint("w"));
    }
}

fn _print_size(metadata: &Metadata) {
    let size = metadata.len();
    let style = Colour::Green.bold();
    if metadata.is_dir() {
        print!("{:<10}", style.paint("-"));
    } else if size > 1024 * 1024 {
        print!("{:<10}", style.paint(format!("{}M", size/1024/1024)));
    } else {
        print!("{:<10}", style.paint(format!("{}", size)));
    }
} 


fn print_last_modified(metadata: &Metadata)  {
    if let Ok(time) = metadata.modified() {
        if let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) {
            let datetime: String = convert_epoch_to_datetime(duration.as_secs(), CEST_UTC_ITALY);
            let style = Colour::Blue.bold();
            print!("{:<10}{}{:>10}","", style.paint(datetime), "");
        } 
    }
}

fn print_filename(metadata: &Metadata, entry: &DirEntry, filename: &str) {
    
    if metadata.is_dir() {
        let style = Colour::Blue.bold();
        print!("{}", style.paint(filename));
    } else if metadata.is_file() {
        print!("{}", filename);
    } else {
        let style = Colour::Blue.bold();
        if let Ok(link) = fs::read_link(entry.path()) {
            print!("{} -> {}", style.paint(filename), link.display());
        } else {
            print!("{} -> can't find linked file", style.paint(filename));
        }
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: u32, year: i32) -> u64 {
    if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[month as usize - 1]
    }
}

fn convert_epoch_to_datetime(epoch: u64, zone: u64) -> String {
    let mut seconds = epoch;
    
    let days = seconds / SECONDS_IN_DAY;
    seconds %= SECONDS_IN_DAY;
    
    let hours = seconds / SECONDS_IN_HOUR;
    seconds %= SECONDS_IN_HOUR;

    let minutes = seconds / SECONDS_IN_MINUTE;

    let mut year = 1970;
    let mut days_remaining = days;
    while days_remaining >= 365 {
        if is_leap_year(year) {
            if days_remaining >= 366 {
                days_remaining -= 366;
                year += 1;
            } else {
                break;
            }
        } else {
            days_remaining -= 365;
            year += 1;
        }
    }

    let mut month = 1;
    while days_remaining >= days_in_month(month, year) {
        days_remaining -= days_in_month(month, year);
        month += 1;
    }
    let day = days_remaining + 1;

    format!("{:02}/{:02}/{:04} {:02}:{:02}", day, month, year, hours + zone, minutes)
}