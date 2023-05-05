#[cfg(test)]
mod test;

use chrono::format::ParseError;
use chrono::{DateTime, FixedOffset};
use std::fmt;

use crate::mergeline::Line;

#[derive(Debug, Clone)]
pub struct LineError;

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line does not have a timestamp")
    }
}

pub fn parse_timestamp_utc(time: &str) -> Result<DateTime<FixedOffset>, LineError> {
    let mut time_utc: String = time.to_string();
    time_utc.push_str(" +0000");
    match DateTime::parse_from_str(time_utc.as_str(), "%Y-%m-%d %H:%M:%S%.f %z") {
        Ok(date_time) => Ok(date_time),
        Err(_) => Err(LineError {}),
    }
}

pub fn get_timestamp_from_line(line: &str) -> Result<String, LineError> {
    let mut chunks = line.split(' ');
    let mut timestamp: String = String::new();

    match chunks.next() {
        Some(text) => timestamp.push_str(text),
        None => return Err(LineError {}),
    }
    timestamp.push(' ');
    match chunks.next() {
        Some(text) => timestamp.push_str(text),
        None => return Err(LineError {}),
    }

    Ok(timestamp)
}
