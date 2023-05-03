use log::*;
use log4rs;
use std::{env, io};

mod app;
use app::run_app;
mod text;
mod timestamp;

#[tokio::main()]
async fn main() -> Result<(), io::Error> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("main - start");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "wrong number of arguments, expected 1 argument = path to a folder to read contents of",
        ));
    }

    run_app(&args[1])?;

    info!("main - end");
    Ok(())
}
