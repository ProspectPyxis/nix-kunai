use crate::source::{
    Source, SourceGetArtifactHashError, SourceMap, SourceMapFromFileJsonError,
    SourceMapWriteToFileError,
};
use clap::Args;
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
    #[error(transparent)]
    ReadSourceFromFileFailed(#[from] SourceMapFromFileJsonError),
    #[error("a source with this name already exists")]
    SourceNameAlreadyExists,
    #[error(transparent)]
    GetArtifactHashError(#[from] SourceGetArtifactHashError),
    #[error(transparent)]
    WriteToFileError(#[from] SourceMapWriteToFileError),
}

pub fn add(source_file_path: &str, args: &AddArgs) -> Result<(), AddError> {
    let mut sources = SourceMap::from_file_json(source_file_path)?;

    if sources.inner.contains_key(&args.source_name) {
        return Err(AddError::SourceNameAlreadyExists);
    }

    let mut new_source = Source::new(&args.initial_version, &args.artifact_url_template);

    new_source.hash = new_source.get_artifact_hash()?;

    sources.inner.insert(args.source_name.clone(), new_source);

    sources.write_to_file(source_file_path)?;

    Ok(())
}
