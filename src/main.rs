use std::{env, io, vec};

use futures::executor::block_on;

mod text;
use text::load_files;
use text::FileWithLines;

mod app;
use app::run_app;

#[tokio::main()]
async fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "wrong number of arguments, expected 1 argument = path to a folder to read contents of",
        ));
    }

    run_app(&args[1])?;

    // testing playground below

    let r = block_on(load_files(vec![
        "testdata\\long-left.log".to_string(),
        "testdata\\long-right.log".to_string(),
    ]));

    let first_file: &FileWithLines = &r[0];

    println!("{}", first_file.get_ith_line(5000));
    println!("{}", first_file.line_breaks.len());

    Ok(())
}
