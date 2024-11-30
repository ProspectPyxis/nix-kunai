use crate::source::SourceMap;
use clap::ValueEnum;
use log::{error, info};
use std::fmt;
use std::process::ExitCode;
use url::Url;

#[derive(Clone, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum EditableSourceKey {
    Pinned,
    ArtifactUrlTemplate,
}

impl fmt::Display for EditableSourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pinned => "pinned",
                Self::ArtifactUrlTemplate => "artifact_url_template",
            }
        )
    }
}

pub fn edit(
    source_file_path: &str,
    source_name: &str,
    source_key: EditableSourceKey,
    value: &str,
) -> ExitCode {
    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let source = match sources.inner.get_mut(source_name) {
        Some(source) => source,
        None => {
            error!("a source named {source_name} does not exist");
            return ExitCode::FAILURE;
        }
    };

    match source_key {
        EditableSourceKey::Pinned => match value.parse() {
            Ok(v) => source.pinned = v,
            Err(_) => {
                error!("invalid value `{value}` for key {source_key} (must be `true` or `false`)");
                return ExitCode::FAILURE;
            }
        },

        EditableSourceKey::ArtifactUrlTemplate => {
            if Url::parse(value).is_err() {
                error!("invalid value `{value}` for key {source_key} (must be a valid URL)");
                return ExitCode::FAILURE;
            }
            source.artifact_url_template = value.to_string();
        }
    }

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        info!("successfully changed value of `{source_key}` to `{value}` in source {source_name}");
        if matches!(source_key, EditableSourceKey::ArtifactUrlTemplate) {
            info!("the changed value could affect the hash; consider running `nix-kunai update --refetch {source_name}`");
        }
        ExitCode::SUCCESS
    }
}
