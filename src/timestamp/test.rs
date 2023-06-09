use super::*;

#[test]
fn timestamp_test() {
    let time = "2023-05-03 10:25:50.262116";
    let parsed = parse_timestamp_utc(time);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().timestamp_micros(), 1683109550262116);
}

#[test]
fn timestamp_test_with_comma() {
    let time = "2023-05-03 10:25:50,262116";
    let parsed = parse_timestamp_utc(time);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().timestamp_micros(), 1683109550262116);
}

#[test]
fn get_from_line_test() {
    let line = "2023-05-03 10:25:50.262116     src\\main.rs INFO  - main - start";
    let parsed = get_timestamp_from_line(line);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap().timestamp_micros(), 1683109550262116);
}
