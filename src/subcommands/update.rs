use crate::source::{get_artifact_hash_from_url, GetArtifactHashError, SourceMap};
use crate::updater::{FetchLatestGitTagError, GetLatestVersionError};
use clap::Args;
use indexmap::IndexMap;
use log::{error, info, warn};
use serde::Serialize;
use std::fmt;
use std::process::ExitCode;

#[derive(Args)]
pub struct UpdateArgs {
    /// Specific sources to update
    #[arg(value_name = "SOURCES")]
    source_names: Vec<String>,
    /// Fetch a new hash even if the version is already latest
    #[arg(short, long)]
    pub refetch: bool,
    /// Force update checking even if the source is pinned
    #[arg(short, long)]
    pub force: bool,
    /// Print updated sources to stdout, regardless of log level
    #[arg(long)]
    pub show_updated: bool,
    /// If any stdout outputs are used, output it as JSON
    #[arg(short, long)]
    pub json: bool,
}

#[derive(Serialize)]
struct VersionDiff {
    pub old: String,
    pub new: String,
}

impl VersionDiff {
    pub fn new(old: String, new: String) -> Self {
        Self { old, new }
    }
}

impl fmt::Display for VersionDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.old == self.new {
            write!(f, "changed hash")
        } else {
            write!(f, "{} -> {}", self.old, self.new)
        }
    }
}

#[derive(Serialize)]
struct UpdatedSources {
    #[serde(flatten)]
    pub inner: IndexMap<String, VersionDiff>,
}

impl UpdatedSources {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::default(),
        }
    }
}

impl Default for UpdatedSources {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update(source_file_path: &str, args: UpdateArgs) -> ExitCode {
    if args.json && !args.show_updated {
        warn!("'--json' was passed, but '--show-updated' is not set");
        warn!("the option will do nothing");
    }

    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let source_filter = args.source_names;

    for source_name in source_filter.iter() {
        if !sources.inner.contains_key(source_name) {
            warn!("source {source_name} does not exist, skipping");
        }
    }

    let mut updated = UpdatedSources::new();
    let mut up_to_date = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for (name, source) in sources
        .inner
        .iter_mut()
        .filter(|(name, _)| source_filter.is_empty() || source_filter.contains(name))
    {
        if source.pinned && !args.force {
            info!("source {name} is pinned; skipping");
            skipped += 1;
            continue;
        }

        info!("checking new versions for source: {name}");
        let latest_tag = match source.update_scheme.get_new_version_for(source) {
            Ok(tag) => tag,
            Err(e) => match e {
                GetLatestVersionError::GetGitUrl(e) => {
                    error!("could not infer git repository url: {e}");
                    error!("git_url may need to be set manually; if so, re-add this source with the correct options");
                    warn!("skipping source {name} with an error");
                    skipped += 1;
                    errors += 1;
                    continue;
                }
                GetLatestVersionError::FetchGitTags {
                    error: FetchLatestGitTagError::NoTagsFitFilter,
                    tag_prefix,
                } => {
                    error!(
                        "no tags found fit the tag prefix `{}`",
                        tag_prefix.as_deref().unwrap_or("")
                    );
                    error!("tag_prefix may be set incorrectly; if so, re-add this source with the correct options");
                    warn!("skipping source {name} with an error");
                    skipped += 1;
                    errors += 1;
                    continue;
                }
                _ => {
                    error!("failed to fetch new version for source {name}: {e}");
                    error!("critical error encountered; aborting update");
                    return ExitCode::FAILURE;
                }
            },
        };

        if !source.update_scheme.is_static()
            && !args.refetch
            && source.latest_checked_version == latest_tag
        {
            info!("{name} is up to date (version {})", source.version);
            up_to_date += 1;
            continue;
        }

        let full_url = match source.full_url(&latest_tag) {
            Ok(url) => url,
            Err(e) => {
                error!("{e}");
                error!("this usually implies that the artifact URL template is broken; fix it or remove the offending source");
                warn!("skipping source {name} with an error");
                skipped += 1;
                errors += 1;
                continue;
            }
        };

        info!(
            "{}fetching hash from {full_url}",
            if source.version == latest_tag && args.refetch {
                "re"
            } else {
                ""
            }
        );
        match get_artifact_hash_from_url(&full_url, source.update_scheme.unpack()) {
            Ok(hash) => {
                if source.version != latest_tag {
                    info!("{name} updated: {} -> {}", source.version, latest_tag);
                    updated.inner.insert(
                        name.to_string(),
                        VersionDiff::new(source.version.clone(), latest_tag.clone()),
                    );
                    source.hash = hash;
                    source.version = latest_tag.clone();
                } else if source.hash != hash {
                    if source.update_scheme.is_static() {
                        info!(
                            "updated hash for source {name} with static version {}",
                            source.version
                        );
                    } else {
                        info!("hash for source {name} changed, but with the same version (version {})", source.version);
                    }
                    updated.inner.insert(
                        name.to_string(),
                        VersionDiff::new(source.version.clone(), latest_tag.clone()),
                    );
                    source.hash = hash;
                } else {
                    info!(
                        "{name} is up to date (same hash) (version {})",
                        source.version
                    );
                    up_to_date += 1;
                }
                source.latest_checked_version = latest_tag;
            }

            Err(e) => match e {
                GetArtifactHashError::PrefetchFailed { .. } => {
                    warn!(
                        "found newer tag {latest_tag} (> {}), but {e}",
                        source.version
                    );
                    warn!("assuming non-release tag; version will not be updated");
                    source.latest_checked_version = latest_tag;
                    up_to_date += 1;
                }
                _ => {
                    error!("unexpected error: {e}");
                    error!("skipping source; the command may have to be rerun");
                    skipped += 1;
                    errors += 1;
                }
            },
        }
    }

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        info!(
            "successfully updated {} source(s) ({skipped} skipped ({errors} with errors), {up_to_date} already up to date)",
            updated.inner.len()
        );

        if args.show_updated {
            if args.json {
                use std::io::{stdout, Write};

                let mut lock = stdout().lock();
                serde_json::to_writer_pretty(&mut lock, &updated).unwrap();
                writeln!(&mut lock).unwrap();
            } else if !updated.inner.is_empty() {
                println!(
                    "Updated packages: {}",
                    updated
                        .inner
                        .iter()
                        .map(|(name, diff)| format!("{} ({})", name, diff))
                        .map(std::borrow::Cow::from)
                        .reduce(|mut acc, s| {
                            acc.to_mut().push_str(", ");
                            acc.to_mut().push_str(&s);
                            acc
                        })
                        .unwrap_or_default()
                )
            }
        }
        ExitCode::SUCCESS
    }
}
