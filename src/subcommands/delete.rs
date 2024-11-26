use crate::source::{SourceMap, SourceMapFromReaderJsonError};
use std::fs::File;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("source file not found at provided path")]
    SourceFileNotFound,
    #[error("could not read source file; permission denied")]
    PermissionDenied,
    #[error("source file json is malformed at line {line}, column {column}")]
    MalformedJson { line: usize, column: usize },
    #[error("source file json does not fit nix-kunai schema at line {line}, column {column}")]
    IncorrectSchema { line: usize, column: usize },
    #[error("a source with that name does not exist")]
    SourceNameNonexistent,
    #[error("could not write new source file; permission denied")]
    WritePermissionDenied,
    #[error("unexpected json error while writing to file: {0}")]
    SerdeWriteError(serde_json::Error),
    #[error("unexpected io error: {0}")]
    Io(io::Error),
}

pub fn delete(source_file_path: &str, source_name: &str) -> Result<(), DeleteError> {
    let source_file = File::open(source_file_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => DeleteError::SourceFileNotFound,
        io::ErrorKind::PermissionDenied => DeleteError::PermissionDenied,
        _ => DeleteError::Io(e),
    })?;

    let mut sources = SourceMap::from_reader_json(source_file).map_err(|e| match e {
        SourceMapFromReaderJsonError::MalformedJson { line, column } => {
            DeleteError::MalformedJson { line, column }
        }
        SourceMapFromReaderJsonError::IncorrectSchema { line, column } => {
            DeleteError::IncorrectSchema { line, column }
        }
        SourceMapFromReaderJsonError::Io(io_err) => DeleteError::Io(io_err),
    })?;

    if sources.inner.remove(source_name).is_none() {
        return Err(DeleteError::SourceNameNonexistent);
    }

    let source_file = File::create(source_file_path).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => DeleteError::WritePermissionDenied,
        _ => DeleteError::Io(e),
    })?;

    serde_json::to_writer_pretty(source_file, &sources).map_err(|e| {
        if let Some(kind) = e.io_error_kind() {
            DeleteError::Io(io::Error::new(kind, e))
        } else {
            DeleteError::SerdeWriteError(e)
        }
    })?;

    Ok(())
}
