use crate::source::Source;
use serde::{Deserialize, Serialize};
use std::io;
use std::num::NonZeroUsize;
use std::process::Command;
use thiserror::Error;
use url::Url;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum VersionUpdateScheme {
    GitTags {
        repo_url: Option<Url>,
        tag_prefix: Option<String>,
    },
    GitBranch {
        repo_url: Option<Url>,
        branch: String,
        short_hash_length: NonZeroUsize,
    },
    Static,
}

#[derive(Debug, Error)]
pub enum GetLatestVersionError {
    #[error("error while getting git repository url: {0}")]
    GetGitUrl(#[from] InferGitUrlError),
    #[error("failed to fetch tags for source: {error}")]
    FetchGitTags {
        error: FetchLatestGitTagError,
        tag_prefix: Option<String>,
    },
    #[error("failed to get commit for branch {branch}: {error}")]
    FetchBranchCommit {
        error: FetchGitBranchCommitError,
        branch: String,
    },
}

impl VersionUpdateScheme {
    pub fn get_new_version_for(&self, source: &Source) -> Result<String, GetLatestVersionError> {
        match self {
            Self::GitTags {
                repo_url,
                tag_prefix,
            } => {
                let git_url = repo_url.as_ref().map_or_else(
                    || infer_git_url(&source.artifact_url_template),
                    |url| Ok(url.clone()),
                )?;

                fetch_latest_git_tag(&git_url, tag_prefix.as_deref()).map_err(|error| {
                    GetLatestVersionError::FetchGitTags {
                        error,
                        tag_prefix: tag_prefix.clone(),
                    }
                })
            }

            Self::GitBranch {
                repo_url,
                branch,
                short_hash_length,
            } => {
                let git_url = repo_url.as_ref().map_or_else(
                    || infer_git_url(&source.artifact_url_template),
                    |url| Ok(url.clone()),
                )?;

                let short_hash = fetch_git_branch_commit(&git_url, branch)
                    .map(|hash| hash[0..(short_hash_length.get())].to_string())
                    .map_err(|error| GetLatestVersionError::FetchBranchCommit {
                        error,
                        branch: branch.clone(),
                    })?;

                Ok(format!("{branch}-{short_hash}"))
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
pub enum InferGitUrlError {
    #[error("could not parse URL template: {0}")]
    CouldNotParseUrlTemplate(#[from] url::ParseError),
    #[error("artifact URL does not have a base")]
    ArtifactUrlNoBase,
    #[error("insufficient path segments to infer URL")]
    InsufficientPathSegments,
}

pub fn infer_git_url(from: &str) -> Result<Url, InferGitUrlError> {
    let mut url = Url::parse(from)?;

    let mut path_segments = url
        .path_segments()
        .ok_or(InferGitUrlError::ArtifactUrlNoBase)?;
    let owner = path_segments
        .next()
        .ok_or(InferGitUrlError::InsufficientPathSegments)?;
    let repo = path_segments
        .next()
        .ok_or(InferGitUrlError::InsufficientPathSegments)?;

    url.set_path(&format!("{owner}/{repo}"));

    Ok(url)
}

#[derive(Debug, Error)]
pub enum FetchGitBranchCommitError {
    #[error("failed to execute command: {full_command}")]
    CommandFailed {
        full_command: String,
        io_error: io::Error,
    },
    #[error("command output is not valid utf8")]
    CommandOutputInvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("could not find the provided branch")]
    BranchNotFound,
}

pub fn fetch_git_branch_commit(
    url: &Url,
    branch: &str,
) -> Result<String, FetchGitBranchCommitError> {
    let args = ["ls-remote", "--branches", url.as_ref()];

    let output = Command::new("git").args(args).output().map_err(|e| {
        FetchGitBranchCommitError::CommandFailed {
            full_command: format!("git {}", args.join(" ")),
            io_error: e,
        }
    })?;

    let output_string = String::from_utf8(output.stdout)?;

    let branch_info = output_string
        .lines()
        .find(|&line| line.split("/").last().is_some_and(|b| b == branch))
        .ok_or(FetchGitBranchCommitError::BranchNotFound)?;

    Ok(branch_info
        .split_whitespace()
        .next()
        .expect("line always starts with a hash")
        .to_string())
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
