use crate::source::{get_artifact_hash_from_url, Source, SourceMap};
use clap::Args;
use log::{error, info};
use std::process::ExitCode;
use url::Url;

#[derive(Args, Clone)]
pub struct AddArgs {
    /// The name of the source
    pub source_name: String,
    /// The url to fetch from for a hash, where {version} replaces the version number
    #[arg(value_name = "ARTIFACT_URL", value_parser = validate_artifact_url)]
    artifact_url_template: String,
    /// Initial version of the package to test for
    #[arg(value_name = "VERSION")]
    initial_version: String,
    /// Prefix to filter tags by
    #[arg(long, value_name = "PREFIX")]
    tag_prefix: Option<String>,
    /// Add the --unpack flag to the prefetch command
    #[arg(short, long)]
    unpack: bool,
    /// Check this git repo instead of inferring from artifact url
    #[arg(long, value_name = "REPOSITORY")]
    git_repo_url: Option<Url>,
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

    if sources.inner.contains_key(&args.source_name) {
        error!("a source called \"{}\" already exists", args.source_name);
        error!(
            "you may be trying to update, or if you want to overwrite the source, delete it first"
        );
        return ExitCode::FAILURE;
    }

    let mut new_source = Source::new(&args.initial_version, &args.artifact_url_template)
        .with_unpack(args.unpack)
        .with_git_url(args.git_repo_url)
        .with_tag_prefix(args.tag_prefix);

    let full_url = match new_source.full_url(&args.initial_version) {
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

    sources.inner.insert(args.source_name.clone(), new_source);

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        info!("added new source \"{}\"", args.source_name);
        ExitCode::SUCCESS
    }
}
