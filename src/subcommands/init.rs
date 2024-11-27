use log::{error, info};
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::process::ExitCode;

pub fn init(source_file_path: &str) -> ExitCode {
    let mut source_file = match File::create_new(source_file_path) {
        Ok(source) => source,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => error!("source file at {source_file_path} already exists"),
                _ => error!("unexpected io error: {e}"),
            }

            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = source_file.write_all("{}".as_bytes()) {
        error!("unexpected io error: {e}");
        ExitCode::FAILURE
    } else {
        info!("successfully created {source_file_path}");
        ExitCode::SUCCESS
    }
}
