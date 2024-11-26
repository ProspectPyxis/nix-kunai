use crate::source::{SourceMap, SourceMapFromFileJsonError, SourceMapWriteToFileError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteError {
    #[error(transparent)]
    ReadSourceFromFileFailed(#[from] SourceMapFromFileJsonError),
    #[error("a source with that name does not exist")]
    SourceNameNonexistent,
    #[error(transparent)]
    WriteToFileError(#[from] SourceMapWriteToFileError),
}

pub fn delete(source_file_path: &str, source_name: &str) -> Result<(), DeleteError> {
    let mut sources = SourceMap::from_file_json(source_file_path)?;

    if sources.inner.remove(source_name).is_none() {
        return Err(DeleteError::SourceNameNonexistent);
    }

    sources.write_to_file(source_file_path)?;

    Ok(())
}
