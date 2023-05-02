use super::*;

#[test]
fn load_file_get_second_line() {
    let text: String = String::from("We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.");
    let line_breaks: Vec<usize> = get_line_breaks(&text);
    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!("It was the spooky slice.", spooky_file.get_ith_line(1));
}
