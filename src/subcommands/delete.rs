use crate::source::SourceMap;
use log::{error, info};
use std::process::ExitCode;

pub fn delete(source_file_path: &str, source_names: Vec<String>) -> ExitCode {
    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    for source_name in &source_names {
        if sources.inner.remove(source_name).is_none() {
            error!("a source named {source_name} does not exist");
            return ExitCode::FAILURE;
        }
    }

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        if source_names.len() == 1 {
            info!("source {} has been removed", source_names[0]);
        } else {
            info!("removed sources: {}", source_names.join(", "));
        }
        ExitCode::SUCCESS
    }
}
