use std::{env, io};

mod text;

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
    Ok(())
}
