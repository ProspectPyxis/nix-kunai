use crate::source::SourceMap;
use log::{error, info};
use std::process::ExitCode;

pub fn delete(source_file_path: &str, source_name: &str) -> ExitCode {
    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    if sources.inner.remove(source_name).is_none() {
        error!("a source named \"{source_name}\" does not exist");
        return ExitCode::FAILURE;
    }

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        info!("source \"{source_name}\" has been removed");
        ExitCode::SUCCESS
    }
}
