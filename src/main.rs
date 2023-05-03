use log::*;
use std::{env, io};

mod app;
use app::run_app;
mod mergeline;
mod text;
mod timestamp;

#[tokio::main()]
async fn main() -> Result<(), io::Error> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("main - start");

    let args: Vec<String> = env::args().collect();

    let path: String = match args.len() {
        1 => std::env::current_dir()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string(),
        2 => args[1].clone(),
        _ => {
            eprintln!("wrong number of arguments, options:\n    1 arguments = read from current working directory\n    1 argument = path to folder to read from");
            return Ok(());
        }
    };

    run_app(&path)?;

    info!("main - end");
    Ok(())
}
