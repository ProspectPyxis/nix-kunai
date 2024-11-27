use crate::source::SourceMap;
use log::{error, info};
use std::process::ExitCode;

pub fn update(source_file_path: &str, source_names: Vec<String>) -> ExitCode {
    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}
