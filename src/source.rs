use log::info;
use serde::{Deserialize, Serialize};
use serde_json::error::Category as JsonErrorCategory;
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::process::Command;
use thiserror::Error;

#[derive(Deserialize, Serialize)]
pub struct Source {
    pub version: String,
    pub latest_checked_version: String,
    pub artifact_url_template: String,
    pub hash: String,
    pub tag_prefix_filter: Option<String>,
    pub unpack: bool,
}

#[derive(Debug, Error)]
pub enum SourceGetArtifactHashError {
    #[error("failed to execute command: {full_command}")]
    CommandFailed {
        full_command: String,
        io_error: io::Error,
    },
    #[error("could not fetch artifact at {full_url}")]
    PrefetchFailed { full_url: String },
    #[error("malformed or incorrect json at line {line}, column {column} of response")]
    MalformedOrIncorrectJson {
        line: usize,
        column: usize,
        response: Vec<u8>,
    },
    #[error("serde failed with an io error: {0}")]
    SerdeIoError(io::Error),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchFileResult {
    hash: String,
}

impl Source {
    pub fn new(version: &str, artifact_url_template: &str) -> Self {
        Source {
            version: version.to_string(),
            latest_checked_version: version.to_string(),
            artifact_url_template: artifact_url_template.to_string(),
            hash: String::new(),
            tag_prefix_filter: None,
            unpack: false,
        }
    }

    pub fn get_artifact_hash(&self) -> Result<String, SourceGetArtifactHashError> {
        let full_url = self
            .artifact_url_template
            .replace("{version}", &self.version);

        let mut args = vec!["store", "prefetch-file", &full_url, "--json"];
        if self.unpack {
            args.push("--unpack");
        }

        info!("fetching artifact from {full_url}");
        let output = Command::new("nix").args(&args).output().map_err(|e| {
            SourceGetArtifactHashError::CommandFailed {
                full_command: format!("nix {}", args.join(" ")),
                io_error: e,
            }
        })?;

        if !output.status.success() {
            return Err(SourceGetArtifactHashError::PrefetchFailed { full_url });
        }

        let response: PrefetchFileResult = serde_json::from_slice(&output.stdout).map_err(|e| {
            if let Some(kind) = e.io_error_kind() {
                SourceGetArtifactHashError::SerdeIoError(io::Error::new(kind, e))
            } else {
                match e.classify() {
                    JsonErrorCategory::Io => {
                        SourceGetArtifactHashError::SerdeIoError(io::Error::other(e))
                    }
                    JsonErrorCategory::Syntax
                    | JsonErrorCategory::Data
                    | JsonErrorCategory::Eof => {
                        SourceGetArtifactHashError::MalformedOrIncorrectJson {
                            line: e.line(),
                            column: e.column(),
                            response: output.stdout,
                        }
                    }
                }
            }
        })?;

        Ok(response.hash)
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct SourceMap {
    #[serde(flatten)]
    pub inner: BTreeMap<String, Source>,
}

#[derive(Debug, Error)]
pub enum SourceMapFromReaderJsonError {
    #[error("unexpected io error: {0}")]
    Io(io::Error),
    #[error("json is malformed at line {line}, column {column}")]
    MalformedJson { line: usize, column: usize },
    #[error("json does not fit nix-kunai schema at line {line}, column {column}")]
    IncorrectSchema { line: usize, column: usize },
}

impl SourceMap {
    pub fn from_reader_json<R: Read>(reader: R) -> Result<Self, SourceMapFromReaderJsonError> {
        serde_json::from_reader(reader).map_err(|e| {
            if let Some(io_error) = e.io_error_kind() {
                SourceMapFromReaderJsonError::Io(io::Error::new(io_error, e))
            } else {
                match e.classify() {
                    JsonErrorCategory::Io => SourceMapFromReaderJsonError::Io(io::Error::other(e)),
                    JsonErrorCategory::Syntax | JsonErrorCategory::Eof => {
                        SourceMapFromReaderJsonError::MalformedJson {
                            line: e.line(),
                            column: e.column(),
                        }
                    }
                    JsonErrorCategory::Data => SourceMapFromReaderJsonError::IncorrectSchema {
                        line: e.line(),
                        column: e.column(),
                    },
                }
            }
        })
    }
}
