use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Check if a year is a leap year
pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Sanitize a string for use in filenames
/// Removes or replaces invalid filesystem characters
pub fn sanitize_filename(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;
    
    for c in s.chars() {
        let mapped = match c {
            ' ' | '_' | '\'' | '"' | ',' | ';' | ':' | '!' | '?' => {
                if !last_was_underscore {
                    last_was_underscore = true;
                    '_'
                } else {
                    continue; // Skip consecutive underscores
                }
            }
            c if c.is_alphanumeric() => {
                last_was_underscore = false;
                c
            }
            '-' | '.' => {
                last_was_underscore = false;
                c
            }
            _ => {
                if !last_was_underscore {
                    last_was_underscore = true;
                    '_'
                } else {
                    continue; // Skip consecutive underscores
                }
            }
        };
        result.push(mapped);
    }
    
    result
        .trim_matches('_') // Remove leading/trailing underscores
        .to_lowercase()
        .chars()
        .take(50) // Limit length
        .collect()
}

/// Get current date in YYYY-MM-DD format
/// Uses environment variable SONG_DATE if set (for testing), otherwise uses current date
pub fn get_current_date() -> String {
    std::env::var("SONG_DATE").unwrap_or_else(|_| {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).unwrap();
        let secs = duration.as_secs();
        
        // Convert seconds to days since epoch
        let days = secs / 86400;
        
        // Calculate year (accounting for leap years)
        let mut year = 1970;
        let mut remaining_days = days;
        
        loop {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }
        
        // Calculate month and day
        let days_in_months = if is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        
        let mut month = 1;
        let mut day = remaining_days as u32;
        
        for &days_in_month in &days_in_months {
            if day < days_in_month {
                break;
            }
            day -= days_in_month;
            month += 1;
        }
        
        day += 1; // Days are 1-indexed
        
        format!("{:04}-{:02}-{:02}", year, month, day)
    })
}

/// Create output directory, returning error if creation fails
pub fn create_output_directory(path: &str) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Could not create output directory: {}", e))
}

