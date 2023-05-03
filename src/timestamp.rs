#[cfg(test)]
mod test;

use chrono::format::ParseError;
use chrono::{DateTime, FixedOffset};
use std::fmt;

#[derive(Debug, Clone)]
pub struct LineError;

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line does not have a timestamp")
    }
}

pub fn parse_timestamp_utc(time: &str) -> Result<DateTime<FixedOffset>, ParseError> {
    let mut time_utc: String = time.to_string();
    time_utc.push_str(" +0000");
    DateTime::parse_from_str(time_utc.as_str(), "%Y-%m-%d %H:%M:%S%.f %z")
}

pub fn get_timestamp_from_line(line: &str) -> Result<String, LineError> {
    let mut chunks = line.split(' ');
    let mut timestamp: String = String::new();

    match chunks.next() {
        Some(text) => timestamp.push_str(text),
        None => return Err(LineError {}),
    }
    timestamp.push_str(" ");
    match chunks.next() {
        Some(text) => timestamp.push_str(text),
        None => return Err(LineError {}),
    }

    return Ok(timestamp);
}
