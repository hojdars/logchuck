use log4rs::append::file;

use super::*;
use std::str::FromStr;

fn generate_line(timestamp: u64, source_file: &str, index: u64) -> Line {
    Line {
        timestamp,
        source_file: String::from_str(source_file).unwrap(),
        index,
    }
}

#[test]
fn cmp_implementation_on_line() {
    assert!(generate_line(7, "2.txt", 0) < generate_line(30, "2.txt", 0));
    assert!(generate_line(30, "123.txt", 10) == generate_line(30, "2.txt", 0));
    assert!(generate_line(50, "2.txt", 0) > generate_line(30, "2.txt", 0));
}

#[test]
fn test_merge() {
    let file_one: Vec<Line> = vec![
        generate_line(0, "1.txt", 0),
        generate_line(10, "1.txt", 1),
        generate_line(15, "1.txt", 2),
        generate_line(30, "1.txt", 3),
    ];

    let file_two: Vec<Line> = vec![
        generate_line(7, "2.txt", 0),
        generate_line(13, "2.txt", 1),
        generate_line(21, "2.txt", 2),
        generate_line(45, "2.txt", 3),
    ];

    let result = merge(&file_one, &file_two);

    assert_eq!(result.len(), file_one.len() + file_two.len());
    assert_eq!(result[0].timestamp, 0);
    assert_eq!(result[1].timestamp, 7);
    assert_eq!(result[2].timestamp, 10);
    assert_eq!(result[3].timestamp, 13);
    assert_eq!(result[4].timestamp, 15);
    assert_eq!(result[5].timestamp, 21);
    assert_eq!(result[6].timestamp, 30);
    assert_eq!(result[7].timestamp, 45);
}
