use super::*;

#[test]
fn load_file_get_ith_line() {
    let text: String = String::from("We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.\n");
    let line_breaks: Vec<usize> = get_line_breaks(&text);

    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!("We did the slice.", spooky_file.get_ith_line(0).unwrap());
    assert_eq!(
        "It was the spooky slice.",
        spooky_file.get_ith_line(1).unwrap()
    );
    assert_eq!("Slice.", spooky_file.get_ith_line(4).unwrap());

    assert_eq!(spooky_file.len(), 5);

    for i in 0..spooky_file.len() {
        assert!(spooky_file.get_ith_line(i).unwrap() != "");
    }
}

#[test]
fn annotated_lines_test() {
    let text: String = String::from("2023-05-03 10:25:50.262116 - one\n2023-05-03 10:25:50.262116 - two\n2023-05-03 10:25:50.262116 - three\n");
    let line_breaks: Vec<usize> = get_line_breaks(&text);
    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!(spooky_file.len(), 3);

    let res = spooky_file.get_annotated_lines(0);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 3);
}

#[test]
fn missing_newline_at_the_end() {
    let text: String = String::from("We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.");
    let line_breaks: Vec<usize> = get_line_breaks(&text);
    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!(spooky_file.len(), 5);
}
