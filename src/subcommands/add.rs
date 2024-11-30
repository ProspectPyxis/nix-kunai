use crate::source::{get_artifact_hash_from_url, Source, SourceMap};
use crate::updater::{
    fetch_latest_git_tag, infer_git_url, FetchLatestGitTagError, InferGitUrlError,
    VersionUpdateScheme,
};
use clap::{Args, Subcommand};
use log::{error, info};
use std::process::ExitCode;
use thiserror::Error;
use url::Url;

#[derive(Args, Clone)]
pub struct AddArgs {
    /// Unpack the artifact,
    /// use this if the artifact link is an archive (.zip, .tar.gz, etc.)
    #[arg(short, long)]
    unpack: bool,
    /// Set the hash to the value provided instead of fetching
    #[arg(long, value_name = "HASH")]
    force_hash: Option<String>,
    /// Mark the source as "pinned", do not update its version
    #[arg(short, long)]
    pinned: bool,
    #[command(subcommand)]
    update_scheme: UpdateSchemeArg,
}

#[derive(Clone, Subcommand)]
pub enum UpdateSchemeArg {
    /// Follow the latest tag in the repository
    GitTags {
        /// The URL to fetch from for a hash,
        /// where {version} will be replaced by the version number
        #[arg(value_parser = validate_artifact_url)]
        artifact_url: String,
        /// Initial version of the package to test for
        /// [default: automatically fetch latest]
        version: Option<String>,
        /// Set source name to this instead of inferring from artifact URL
        #[arg(short = 'n', long)]
        source_name: Option<String>,
        /// Check latest tags from this repository URL
        /// instead of inferring from artifact URL
        #[arg(long, value_name = "REPOSITORY")]
        git_repo: Option<Url>,
        /// Prefix to filter tags by
        #[arg(long, value_name = "PREFIX")]
        tag_prefix: Option<String>,
    },
    /// Don't change the version, only the hash
    Static {
        /// Name of the source
        #[arg(value_name = "NAME")]
        source_name: String,
        /// The URL to fetch from for a hash,
        /// where {version} will be replaced by the version number
        #[arg(value_parser = validate_artifact_url)]
        artifact_url: String,
        /// String to use as a "version"
        version: String,
    },
}

fn validate_artifact_url(s: &str) -> Result<String, String> {
    Url::parse(s).map_err(|e| e.to_string())?;

    Ok(s.to_string())
}

pub fn add(source_file_path: &str, args: AddArgs) -> ExitCode {
    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let source_name = match build_source_name(&args.update_scheme) {
        Ok(name) => name,
        Err(SourceNameError::GetGitUrlFailed(e)) => {
            error!("could not infer git repository URL from artifact URL: {e}");
            error!("define '--git-repo' manually");
            return ExitCode::FAILURE;
        }
    };

    if sources.inner.contains_key(&source_name) {
        if matches!(
            args.update_scheme,
            UpdateSchemeArg::GitTags {
                source_name: None,
                ..
            }
        ) {
            error!("source name was inferred as '{source_name}', but said source already exists");
            error!("define '--name' manually");
        } else {
            error!("a source called {source_name} already exists");
            error!(
                "you may be trying to update, or if you want to overwrite the source, delete it first"
            );
        }
        return ExitCode::FAILURE;
    }

    let initial_version = match build_initial_version(&args.update_scheme) {
        Ok(v) => v,
        Err(e) => {
            match e {
                InitialVersionError::GetGitUrlFailed(e) => {
                    error!("could not infer git repository URL from artifact URL: {e}");
                    error!("define '--git-repo' manually");
                }
                InitialVersionError::NoTagsFitPrefix(prefix) => {
                    error!(
                        "no tags fit the tag prefix {}",
                        match prefix {
                            Some(prefix) => format!("'{prefix}'"),
                            None => "(none)".to_string(),
                        }
                    );
                }
                _ => error!("{e}"),
            };
            return ExitCode::FAILURE;
        }
    };

    let mut new_source = build_source(&args.update_scheme, &initial_version)
        .with_unpack(args.unpack)
        .with_pinned(args.pinned);

    if let Some(hash) = args.force_hash {
        new_source.hash = hash;
    } else {
        let full_url = match new_source.full_url(&initial_version) {
            Ok(url) => url,
            Err(e) => {
                error!("{e}");
                return ExitCode::FAILURE;
            }
        };
        info!("fetching hash from {full_url}");
        new_source.hash = match get_artifact_hash_from_url(&full_url, args.unpack) {
            Ok(hash) => hash,
            Err(e) => {
                error!("{e}");
                return ExitCode::FAILURE;
            }
        };
    }

    sources.inner.insert(source_name.clone(), new_source);

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        info!("added new source {}", source_name);
        ExitCode::SUCCESS
    }
}

#[derive(Debug, Error)]
enum SourceNameError {
    #[error("could not infer git repository url: {0}")]
    GetGitUrlFailed(#[from] InferGitUrlError),
}

fn build_source_name(update_scheme: &UpdateSchemeArg) -> Result<String, SourceNameError> {
    match update_scheme {
        UpdateSchemeArg::GitTags {
            artifact_url,
            source_name,
            git_repo,
            ..
        } => {
            let git_url = git_repo
                .clone()
                .map_or_else(|| infer_git_url(artifact_url), Ok)?;

            Ok(source_name.clone().unwrap_or_else(|| {
                git_url
                    .path_segments()
                    .expect("git url must be base")
                    .last()
                    .expect("inferred git url must have at least two path segments")
                    .to_string()
            }))
        }

        UpdateSchemeArg::Static { source_name, .. } => Ok(source_name.clone()),
    }
}

#[derive(Debug, Error)]
enum InitialVersionError {
    #[error("could not infer git repository url: {0}")]
    GetGitUrlFailed(#[from] InferGitUrlError),
    #[error("no tags found that fit the tag prefix")]
    NoTagsFitPrefix(Option<String>),
    #[error("could not fetch latest tag from {git_url}: {error}")]
    FetchTagsFailed {
        git_url: Url,
        error: Box<FetchLatestGitTagError>,
    },
}

fn build_initial_version(update_scheme: &UpdateSchemeArg) -> Result<String, InitialVersionError> {
    match update_scheme {
        UpdateSchemeArg::GitTags {
            artifact_url,
            version,
            git_repo,
            tag_prefix,
            ..
        } => {
            let git_url = git_repo
                .as_ref()
                .map_or_else(|| infer_git_url(artifact_url), |url| Ok(url.clone()))?;

            version.clone().map_or_else(
                || {
                    fetch_latest_git_tag(&git_url, tag_prefix.as_deref()).map_err(|e| match e {
                        FetchLatestGitTagError::NoTagsFitFilter => {
                            InitialVersionError::NoTagsFitPrefix(tag_prefix.clone())
                        }
                        _ => InitialVersionError::FetchTagsFailed {
                            git_url,
                            error: Box::new(e),
                        },
                    })
                },
                Ok,
            )
        }
        UpdateSchemeArg::Static { version, .. } => Ok(version.clone()),
    }
}

fn build_source(update_scheme: &UpdateSchemeArg, version: &str) -> Source {
    match update_scheme {
        UpdateSchemeArg::GitTags {
            artifact_url,
            git_repo,
            tag_prefix,
            ..
        } => {
            let update_scheme = VersionUpdateScheme::GitTags {
                repo_url: git_repo.clone(),
                tag_prefix: tag_prefix.clone(),
            };

            Source::new(version, artifact_url, update_scheme)
        }

        UpdateSchemeArg::Static { artifact_url, .. } => {
            Source::new(version, artifact_url, VersionUpdateScheme::Static)
        }
    }
}
