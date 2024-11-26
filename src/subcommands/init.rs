use std::fs::File;
use std::io::{ErrorKind, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("source file already exists")]
    SourceFileExists,
    #[error("unexpected io error: {0}")]
    Io(std::io::Error),
}

pub fn init(source_file_path: &str) -> Result<(), InitError> {
    let mut source_file = File::create_new(source_file_path).map_err(|e| match e.kind() {
        ErrorKind::AlreadyExists => InitError::SourceFileExists,
        _ => InitError::Io(e),
    })?;

    source_file
        .write_all("{}".as_bytes())
        .map_err(InitError::Io)?;

    Ok(())
}
