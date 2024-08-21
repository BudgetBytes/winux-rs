use std::fs::{self, Metadata, DirEntry};

const RESET: &str = "\x1B[0m";
const BLUE: &str = "\x1B[34m";
const _BLACK: &str = "\x1B[30m";
const _RED: &str = "\x1B[31m";
const _GREEN: &str = "\x1B[32m";
const _YELLOW: &str = "\x1B[33m";
const _MAGENTA: &str = "\x1B[35m";
const _CYAN: &str = "\x1B[36m";
const _WHITE: &str = "\x1B[37m";

const BRIGHT_RED: &str = "\x1B[91m";
const BRIGHT_GREEN: &str = "\x1B[92m";
const BRIGHT_YELLOW: &str = "\x1B[93m";
const BRIGHT_BLUE: &str = "\x1B[94m";
const BRIGHT_CYAN: &str = "\x1B[96m";
const _BRIGHT_BLACK: &str = "\x1B[90m";
const _BRIGHT_MAGENTA: &str = "\x1B[95m";
const _BRIGHT_WHITE: &str = "\x1B[97m";

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
                        println!("{}{}{:>20}    {:>30}    {}", format_type(&metadata), format_permissions(&metadata), format_size(&metadata), format_last_modified(&metadata), format_filename(&metadata, &entry, filename));
                    }
                } else {
                    println!("Couldn't get metadata for {:?}", entry.path());
                }
            }
        }
    }
}

fn format_last_modified(metadata: &Metadata) -> String {
    if let Ok(time) = metadata.modified() {
        if let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) {
            let datetime: String = convert_epoch_to_datetime(duration.as_secs(), CEST_UTC_ITALY);
            return format!("{}{}{}", BLUE, datetime, RESET);
        } else {
            return format!("");
        }
    } else {
        return format!("");
    }
}

fn format_size(metadata: &Metadata) -> String {
    let size = metadata.len();
    if metadata.is_dir() {
        return format!("{}{}{}", BRIGHT_GREEN, "-", RESET);
    }
    if size > 1024 * 1024 {
        return format!("{}{}M{}", BRIGHT_GREEN, size/1024/1024, RESET);
    } else {
        return format!("{}{}{}", BRIGHT_GREEN, size, RESET);
    }
} 

fn format_type(metadata: &Metadata) -> String {
    if metadata.is_dir() {
        return format!("{}{}{}",BRIGHT_BLUE, "d", RESET); 
    } else if metadata.is_file() {
        return format!("{}", "-"); 
    } else {
        return format!("{}{}{}", BRIGHT_CYAN, "l", RESET); 
    }
}

fn format_filename(metadata: &Metadata, entry: &DirEntry, filename: &str) -> String {
    
    if metadata.is_dir() {
        return format!("{}{}{}",BRIGHT_BLUE, filename, RESET); 
    } else if metadata.is_file() {
        return format!("{}{}{}", RESET, filename, RESET); 
    } else {
        if let Ok(link) = fs::read_link(entry.path()) {
            return format!("{}{}{} -> {}", BRIGHT_CYAN, filename, RESET, link.display());
        } else {
            return format!("{}{}{} -> can't find linked file", BRIGHT_CYAN, filename, RESET); 
        }
    }
}

fn format_permissions(metadata: &Metadata) -> String {
    if metadata.permissions().readonly() {
        return format!("{}{}{}{}{}", BRIGHT_YELLOW, "r", BRIGHT_RED, "-", RESET);
    } else {
        return format!("{}{}{}{}{}", BRIGHT_YELLOW, "r", BRIGHT_RED, "w", RESET);
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