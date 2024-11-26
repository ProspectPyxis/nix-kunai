use crate::source::{Source, SourceGetArtifactHashError, SourceMap, SourceMapFromReaderJsonError};
use clap::Args;
use std::fs::File;
use std::io;
use thiserror::Error;

#[derive(Args, Clone)]
pub struct AddArgs {
    /// The name of the source
    pub source_name: String,
    /// The url to fetch from for a hash, where {version} replaces the version number
    #[arg(value_name = "ARTIFACT_URL")]
    artifact_url_template: String,
    /// Initial version of the package to test for
    #[arg(value_name = "VERSION")]
    initial_version: String,
    /// Add the --unpack flag to the prefetch command
    #[arg(short, long)]
    unpack: bool,
    /// Check this git repo instead of inferring from artifact url
    #[arg(long, value_name = "REPOSITORY")]
    git_repo_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("source file not found at provided path")]
    SourceFileNotFound,
    #[error("could not read source file; permission denied")]
    PermissionDenied,
    #[error("a source with this name already exists")]
    SourceNameAlreadyExists,
    #[error("source file json is malformed at line {line}, column {column}")]
    MalformedJson { line: usize, column: usize },
    #[error("source file json does not fit nix-kunai schema at line {line}, column {column}")]
    IncorrectSchema { line: usize, column: usize },
    #[error(transparent)]
    GetArtifactHashError(#[from] SourceGetArtifactHashError),
    #[error("unexpected io error: {0}")]
    Io(io::Error),
    #[error("unexpected json error while writing to file: {0}")]
    SerdeWriteError(serde_json::Error),
}

pub fn add(source_file_path: &str, args: &AddArgs) -> Result<(), AddError> {
    let source_file = File::open(source_file_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => AddError::SourceFileNotFound,
        io::ErrorKind::PermissionDenied => AddError::PermissionDenied,
        _ => AddError::Io(e),
    })?;

    let mut sources = SourceMap::from_reader_json(source_file).map_err(|e| match e {
        SourceMapFromReaderJsonError::MalformedJson { line, column } => {
            AddError::MalformedJson { line, column }
        }
        SourceMapFromReaderJsonError::IncorrectSchema { line, column } => {
            AddError::IncorrectSchema { line, column }
        }
        SourceMapFromReaderJsonError::Io(io_err) => AddError::Io(io_err),
    })?;

    if sources.inner.contains_key(&args.source_name) {
        return Err(AddError::SourceNameAlreadyExists);
    }

    let mut new_source = Source::new(&args.initial_version, &args.artifact_url_template);

    new_source.hash = new_source.get_artifact_hash()?;

    sources.inner.insert(args.source_name.clone(), new_source);

    let source_file = File::create(source_file_path).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => AddError::PermissionDenied,
        _ => AddError::Io(e),
    })?;

    serde_json::to_writer_pretty(source_file, &sources).map_err(|e| {
        if let Some(kind) = e.io_error_kind() {
            AddError::Io(io::Error::new(kind, e))
        } else {
            AddError::SerdeWriteError(e)
        }
    })?;

    Ok(())
}
