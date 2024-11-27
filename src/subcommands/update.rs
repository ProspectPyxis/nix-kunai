use crate::source::{
    fetch_latest_git_tag, get_artifact_hash_from_url, GetArtifactHashError, SourceMap,
};
use log::{error, info, warn};
use std::process::ExitCode;

pub fn update(source_file_path: &str, source_filter: Vec<String>) -> ExitCode {
    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    for source_name in source_filter.iter() {
        if !sources.inner.contains_key(source_name) {
            warn!("source \"{source_name}\" does not exist, skipping");
        }
    }

    for (name, source) in sources
        .inner
        .iter_mut()
        .filter(|(name, _)| source_filter.is_empty() || source_filter.contains(name))
    {
        let git_url = match source
            .git_url(true)
            .expect("should never be none thanks to infer = true")
        {
            Ok(url) => url,
            Err(e) => {
                error!("error while getting git URL of source {name}: {e}");
                error!("skipping; you may have to manually add a git repo URL or fix the existing link");
                continue;
            }
        };

        info!("checking new versions for source: {name}");
        let latest_tag = match fetch_latest_git_tag(&git_url, source.tag_prefix_filter.as_deref()) {
            Ok(tag) => tag,
            Err(e) => {
                error!("failed to fetch tags for source {name}: {e}");
                error!("critical error encountered; aborting update");
                return ExitCode::FAILURE;
            }
        };

        if source.latest_checked_version == latest_tag {
            info!("{name} is up to date ({latest_tag})");
            continue;
        }

        let full_url = match source.full_url(&latest_tag) {
            Ok(url) => url,
            Err(e) => {
                error!("{e}");
                error!("this usually implies that the URL template is broken; fix it or remove the offending source");
                return ExitCode::FAILURE;
            }
        };

        info!("fetching hash from {full_url}");
        match get_artifact_hash_from_url(&full_url, source.unpack) {
            Ok(hash) => {
                info!("{name} updated: {} -> {}", source.version, latest_tag);
                source.hash = hash;
                source.version = latest_tag.clone();
                source.latest_checked_version = latest_tag;
            }
            Err(e) => match e {
                GetArtifactHashError::PrefetchFailed { .. } => {
                    warn!("{e}");
                    warn!(
                        "assuming non-release tag; version will not be updated {}",
                        source.version
                    );
                    source.latest_checked_version = latest_tag;
                }
                _ => {
                    error!("unexpected error: {e}");
                    error!("skipping source; the command may have to be rerun");
                }
            },
        }
    }

    ExitCode::SUCCESS
}
