use crate::source::{InferGitUrlError, Source};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::io;
use std::process::Command;
use thiserror::Error;
use url::Url;

#[derive(Clone, Copy, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum VersionUpdateScheme {
    GitTags,
    Static,
}

#[derive(Debug, Error)]
pub enum GetLatestVersionError {
    #[error("error while getting git repository url: {0}")]
    GetGitUrlFailed(#[from] InferGitUrlError),
    #[error("failed to fetch tags for source: {0}")]
    FetchGitTagsFailed(#[from] FetchLatestGitTagError),
}

impl VersionUpdateScheme {
    pub fn get_new_version_for(&self, source: &Source) -> Result<String, GetLatestVersionError> {
        match self {
            Self::GitTags => {
                let git_url = source
                    .git_url(true)
                    .expect("should never be None thanks to infer = true")?;

                Ok(fetch_latest_git_tag(
                    &git_url,
                    source.tag_prefix_filter.as_deref(),
                )?)
            }

            Self::Static => Ok(source.version.clone()),
        }
    }

    // Static is generally just a huge edge case, so it should be easy to check
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static)
    }
}

#[derive(Debug, Error)]
pub enum FetchLatestGitTagError {
    #[error("failed to execute command: {full_command}")]
    CommandFailed {
        full_command: String,
        io_error: io::Error,
    },
    #[error("command output is not valid utf8")]
    CommandOutputInvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("no tag fits the provided filter")]
    NoTagsFitFilter,
}

pub fn fetch_latest_git_tag(
    url: &Url,
    filter: Option<&str>,
) -> Result<String, FetchLatestGitTagError> {
    let args = [
        "-c",
        "versionsort.suffix=-",
        "ls-remote",
        "--tags",
        "--sort=v:refname",
        url.as_ref(),
    ];

    let output = Command::new("git").args(args).output().map_err(|e| {
        FetchLatestGitTagError::CommandFailed {
            full_command: format!("git {}", args.join(" ")),
            io_error: e,
        }
    })?;

    let output_string = String::from_utf8(output.stdout)?;

    let filter = filter.unwrap_or("");
    let latest_tag = output_string
        .lines()
        .map(|line| line.split('/').last().unwrap_or(""))
        .filter(|line| {
            line.starts_with(filter)
                && line
                    .chars()
                    .nth(filter.len())
                    .is_some_and(|c| c.is_ascii_digit())
        })
        .last();

    let latest_tag = match latest_tag {
        Some(tag) => tag,
        None => return Err(FetchLatestGitTagError::NoTagsFitFilter),
    };

    Ok(latest_tag
        .strip_prefix(filter)
        .unwrap_or(latest_tag)
        .to_string())
}
