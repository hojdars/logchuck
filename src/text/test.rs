use super::*;

#[test]
fn load_file_get_second_line() {
    let text: String = String::from("We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.\n");
    let line_breaks: Vec<usize> = get_line_breaks(&text);

    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!("We did the slice.", spooky_file.get_ith_line(0));
    assert_eq!("It was the spooky slice.", spooky_file.get_ith_line(1));
    assert_eq!("Slice.", spooky_file.get_ith_line(4));

    assert_eq!(spooky_file.len(), 5);

    for i in 0..spooky_file.len() {
        assert!(spooky_file.get_ith_line(i) != "");
    }
}

#[test]
fn load_file_get_lines() {
    let text: String = String::from("We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.\n");
    let line_breaks: Vec<usize> = get_line_breaks(&text);
    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!(spooky_file.len(), 5);

    let res = spooky_file.get_lines(0, spooky_file.len());
    println!("{:?}", res);
    assert_eq!(res.len(), 5);

    assert_eq!(res[0], "We did the slice.");
    assert_eq!(res[1], "It was the spooky slice.");
    assert_eq!(res[4], "Slice.");
}

#[test]
fn load_file_get_lines_out_of_bounds() {
    let text: String = String::from("We did the slice.\nIt was the spooky slice.\nNow our swings have some spice.\nSpoooooky.\nSlice.\n");
    let line_breaks: Vec<usize> = get_line_breaks(&text);
    let spooky_file = FileWithLines { text, line_breaks };
    assert_eq!(spooky_file.len(), 5);

    let res = spooky_file.get_lines(0, 137);
    assert_eq!(res.len(), 5);

    let res = spooky_file.get_lines(0, 0);
    assert_eq!(res.len(), 0);

    let res = spooky_file.get_lines(2, 0);
    assert_eq!(res.len(), 0);

    let res = spooky_file.get_lines(9, 10);
    assert_eq!(res.len(), 0);
}
