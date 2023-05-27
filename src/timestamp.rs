#[cfg(test)]
mod test;

use chrono::{DateTime, FixedOffset};
use std::fmt;

#[derive(Debug, Clone)]
pub struct LineError {
    pub error_message: String,
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

pub fn get_timestamp_from_line(line: &str) -> Result<DateTime<FixedOffset>, LineError> {
    let mut chunks = line.split(' ');
    let mut timestamp: String = String::new();

    match chunks.next() {
        Some(text) => timestamp.push_str(text),
        None => {
            return Err(LineError {
                error_message: String::from("line does not have a date"),
            })
        }
    }
    timestamp.push(' ');
    match chunks.next() {
        Some(text) => timestamp.push_str(text),
        None => {
            return Err(LineError {
                error_message: String::from("line does not have a time"),
            })
        }
    }

    return parse_timestamp_utc(timestamp.as_str());
}

fn parse_timestamp_utc(time: &str) -> Result<DateTime<FixedOffset>, LineError> {
    let mut time_utc: String = time.to_string();
    time_utc.push_str(" +0000");

    if let Ok(date_time) = DateTime::parse_from_str(time_utc.as_str(), "%Y-%m-%d %H:%M:%S%.f %z") {
        return Ok(date_time);
    }

    if let Ok(date_time) = DateTime::parse_from_str(time_utc.as_str(), "%Y-%m-%d %H:%M:%S%.f %z") {
        return Ok(date_time);
    }

    time_utc = time_utc.replace(',', ".");
    if let Ok(date_time) = DateTime::parse_from_str(time_utc.as_str(), "%Y-%m-%d %H:%M:%S%.f %z") {
        return Ok(date_time);
    }

    Err(LineError {
        error_message: String::from("cannot parse timestamp"),
    })
}
