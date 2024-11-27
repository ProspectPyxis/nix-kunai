use crate::updater::VersionUpdateScheme;
use serde::{Deserialize, Serialize};
use serde_json::error::Category as JsonErrorCategory;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use thiserror::Error;
use url::Url;

#[derive(Deserialize, Serialize)]
pub struct Source {
    pub version: String,
    pub latest_checked_version: String,
    pub artifact_url_template: String,
    #[serde(rename = "git_url")]
    git_url_inner: Option<Url>,
    pub hash: String,
    pub tag_prefix_filter: Option<String>,
    pub unpack: bool,
    pub update_scheme: VersionUpdateScheme,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchFileResult {
    hash: String,
}

#[derive(Debug, Error)]
#[error("constructed full URL {full_url} is invalid: {parse_error}")]
pub struct BuildFullUrlError {
    full_url: String,
    parse_error: url::ParseError,
}

#[derive(Debug, Error)]
pub enum InferGitUrlError {
    #[error("could not parse URL template: {0}")]
    CouldNotParseUrlTemplate(#[from] url::ParseError),
    #[error("artifact URL does not have a base")]
    ArtifactUrlNoBase,
    #[error("insufficient path segments to infer URL")]
    InsufficientPathSegments,
}

impl Source {
    pub fn new(
        version: &str,
        artifact_url_template: &str,
        update_scheme: VersionUpdateScheme,
    ) -> Self {
        Source {
            version: version.to_string(),
            latest_checked_version: version.to_string(),
            artifact_url_template: artifact_url_template.to_string(),
            git_url_inner: None,
            hash: String::new(),
            tag_prefix_filter: None,
            unpack: false,
            update_scheme,
        }
    }

    pub fn with_unpack(self, unpack: bool) -> Self {
        Self { unpack, ..self }
    }

    pub fn with_git_url(self, git_url: Option<Url>) -> Self {
        Self {
            git_url_inner: git_url,
            ..self
        }
    }

    pub fn with_tag_prefix(self, tag_prefix_filter: Option<String>) -> Self {
        Self {
            tag_prefix_filter,
            ..self
        }
    }

    pub fn git_url(&self, infer: bool) -> Option<Result<Url, InferGitUrlError>> {
        if let Some(url) = &self.git_url_inner {
            Some(Ok(url.clone()))
        } else if infer {
            let mut url = match Url::parse(&self.artifact_url_template) {
                Ok(url) => url,
                Err(e) => return Some(Err(e.into())),
            };

            let mut path_segments = match url.path_segments() {
                Some(segments) => segments,
                None => return Some(Err(InferGitUrlError::ArtifactUrlNoBase)),
            };
            let owner = match path_segments.next() {
                Some(segment) => segment,
                None => return Some(Err(InferGitUrlError::InsufficientPathSegments)),
            };
            let repo = match path_segments.next() {
                Some(segment) => segment,
                None => return Some(Err(InferGitUrlError::InsufficientPathSegments)),
            };

            url.set_path(&format!("{owner}/{repo}"));

            Some(Ok(url))
        } else {
            None
        }
    }

    pub fn full_url(&self, version: &str) -> Result<Url, BuildFullUrlError> {
        let full_url = self.artifact_url_template.replace("{version}", version);

        Url::parse(&full_url).map_err(|parse_error| BuildFullUrlError {
            full_url,
            parse_error,
        })
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct SourceMap {
    #[serde(flatten)]
    pub inner: BTreeMap<String, Source>,
}

#[derive(Debug, Error)]
pub enum SourceMapFromFileJsonError {
    #[error("source file does not exist")]
    NotFound,
    #[error("could not read source file; permission denied")]
    PermissionDenied,
    #[error("source file json is malformed at line {line}, column {column}")]
    MalformedJson { line: usize, column: usize },
    #[error(
        "source file json does not confirm to nix-kunai schema at line {line}, column {column}"
    )]
    IncorrectSchema { line: usize, column: usize },
    #[error("unexpected io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum SourceMapWriteToFileError {
    #[error("could not write to source file; permission denied")]
    PermissionDenied,
    #[error("unexpected io error: {0}")]
    Io(io::Error),
    #[error("unexpected json error while writing to source file: {0}")]
    SerdeWriteError(serde_json::Error),
}

impl SourceMap {
    pub fn from_reader_json<R: Read>(reader: R) -> Result<Self, serde_json::Error> {
        serde_json::from_reader(reader)
    }

    pub fn from_file_json<P: AsRef<Path>>(path: P) -> Result<Self, SourceMapFromFileJsonError> {
        let file = File::open(path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => SourceMapFromFileJsonError::NotFound,
            io::ErrorKind::PermissionDenied => SourceMapFromFileJsonError::PermissionDenied,
            _ => SourceMapFromFileJsonError::Io(e),
        })?;

        Self::from_reader_json(file).map_err(|e| {
            if let Some(kind) = e.io_error_kind() {
                io::Error::new(kind, e).into()
            } else {
                match e.classify() {
                    JsonErrorCategory::Io => io::Error::other(e).into(),
                    JsonErrorCategory::Syntax | JsonErrorCategory::Eof => {
                        SourceMapFromFileJsonError::MalformedJson {
                            line: e.line(),
                            column: e.column(),
                        }
                    }
                    JsonErrorCategory::Data => SourceMapFromFileJsonError::IncorrectSchema {
                        line: e.line(),
                        column: e.column(),
                    },
                }
            }
        })
    }

    pub fn write_to_writer_pretty<W: Write>(&self, writer: W) -> Result<(), serde_json::Error> {
        serde_json::to_writer_pretty(writer, self)
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SourceMapWriteToFileError> {
        let file = File::create(path).map_err(|e| match e.kind() {
            io::ErrorKind::PermissionDenied => SourceMapWriteToFileError::PermissionDenied,
            _ => SourceMapWriteToFileError::Io(e),
        })?;

        self.write_to_writer_pretty(file)
            .map_err(SourceMapWriteToFileError::SerdeWriteError)
    }
}

#[derive(Debug, Error)]
pub enum GetArtifactHashError {
    #[error("failed to execute command: {full_command}")]
    CommandFailed {
        full_command: String,
        io_error: io::Error,
    },
    #[error("could not fetch artifact at {url}")]
    PrefetchFailed { url: String },
    #[error("malformed or incorrect json at line {line}, column {column} of response")]
    MalformedOrIncorrectJson {
        line: usize,
        column: usize,
        response: Vec<u8>,
    },
    #[error("serde failed with an io error: {0}")]
    SerdeIoError(io::Error),
}

pub fn get_artifact_hash_from_url(url: &Url, unpack: bool) -> Result<String, GetArtifactHashError> {
    let url_string = url.to_string();
    let mut args = vec!["store", "prefetch-file", &url_string, "--json"];
    if unpack {
        args.push("--unpack");
    }

    let output = Command::new("nix").args(&args).output().map_err(|e| {
        GetArtifactHashError::CommandFailed {
            full_command: format!("nix {}", args.join(" ")),
            io_error: e,
        }
    })?;

    if !output.status.success() {
        return Err(GetArtifactHashError::PrefetchFailed {
            url: url.to_string(),
        });
    }

    let response: PrefetchFileResult = serde_json::from_slice(&output.stdout).map_err(|e| {
        if let Some(kind) = e.io_error_kind() {
            GetArtifactHashError::SerdeIoError(io::Error::new(kind, e))
        } else {
            match e.classify() {
                JsonErrorCategory::Io => GetArtifactHashError::SerdeIoError(io::Error::other(e)),
                JsonErrorCategory::Syntax | JsonErrorCategory::Data | JsonErrorCategory::Eof => {
                    GetArtifactHashError::MalformedOrIncorrectJson {
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
