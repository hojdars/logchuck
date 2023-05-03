use super::*;

fn generate_line(timestamp: i64, source_file: usize, index: usize) -> Line {
    Line {
        timestamp,
        source_file,
        index,
    }
}

#[test]
fn cmp_implementation_on_line() {
    assert!(generate_line(7, 1, 0) < generate_line(30, 1, 0));
    assert!(generate_line(30, 1337, 10) == generate_line(30, 1, 0));
    assert!(generate_line(50, 1, 0) > generate_line(30, 1, 0));
}

#[test]
fn test_merge() {
    let file_one: Vec<Line> = vec![
        generate_line(0, 0, 0),
        generate_line(10, 0, 1),
        generate_line(15, 0, 2),
        generate_line(30, 0, 3),
    ];

    let file_two: Vec<Line> = vec![
        generate_line(7, 1, 0),
        generate_line(13, 1, 1),
        generate_line(21, 1, 2),
        generate_line(45, 1, 3),
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
