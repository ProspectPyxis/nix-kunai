use crate::source::{get_artifact_hash_from_url, Source, SourceMap};
use crate::updater::VersionUpdateScheme;
use crate::Cli;
use clap::Args;
use log::{error, info};
use std::process::ExitCode;
use url::Url;

#[derive(Args, Clone)]
pub struct AddArgs {
    /// The name of the source
    pub source_name: String,
    /// The url to fetch from for a hash, where {version} will be replaced by the version number
    #[arg(value_name = "ARTIFACT_URL", value_parser = validate_artifact_url)]
    artifact_url_template: String,
    /// Initial version of the package to test for
    #[arg(value_name = "VERSION")]
    initial_version: String,
    /// Add the --unpack flag to the prefetch command
    #[arg(short, long)]
    unpack: bool,
    /// Set the hash to the value provided instead of fetching
    #[arg(long, value_name = "HASH")]
    force_hash: Option<String>,
    /// Mark the source as "pinned", do not update its version
    #[arg(short, long)]
    pinned: bool,
    /// The version update scheme to use for this source
    #[arg(long, value_enum, value_name = "SCHEME", default_value_t = VersionUpdateScheme::GitTags)]
    update_scheme: VersionUpdateScheme,
    /// Prefix to filter tags by
    #[arg(
        help_heading = Some("Options for --update-scheme git-tags"),
        long, value_name = "PREFIX"
    )]
    tag_prefix: Option<String>,
    /// Check this git repo instead of inferring from artifact url
    #[arg(
        help_heading = Some("Options for --update-scheme git-tags"),
        long, value_name = "REPOSITORY"
    )]
    git_repo_url: Option<Url>,
}

fn validate_artifact_url(s: &str) -> Result<String, String> {
    Url::parse(s).map_err(|e| e.to_string())?;

    Ok(s.to_string())
}

fn validate_add_args(args: &AddArgs) {
    use clap::error::{Error, ErrorKind, RichFormatter};
    use clap::CommandFactory;

    let make_arg_conflict_error = |msg| {
        let cmd = Cli::command();
        // We use this custom message and a raw error here
        // because otherwise clap will show the base command usage,
        // making it just a little less seamless
        let msg = format!(
            "{msg}\n\n{}\n",
            color_print::cstr!("For more information, try <bold>'--help'</bold>.")
        );
        Error::<RichFormatter>::raw(ErrorKind::ArgumentConflict, msg).with_cmd(&cmd)
    };

    if !matches!(args.update_scheme, VersionUpdateScheme::GitTags) {
        let error_str = format!(
            "{} can't be specified without --update-scheme git-tags",
            match args {
                _ if args.tag_prefix.is_some() => "--tag-prefix",
                _ if args.git_repo_url.is_some() => "--git-repo-url",
                _ => return,
            },
        );

        make_arg_conflict_error(error_str).exit();
    }
}

pub fn add(source_file_path: &str, args: AddArgs) -> ExitCode {
    validate_add_args(&args);

    let mut sources = match SourceMap::from_file_json(source_file_path) {
        Ok(s) => s,
        Err(e) => {
            error!("{e}");
            return ExitCode::FAILURE;
        }
    };

    if sources.inner.contains_key(&args.source_name) {
        error!("a source called {} already exists", args.source_name);
        error!(
            "you may be trying to update, or if you want to overwrite the source, delete it first"
        );
        return ExitCode::FAILURE;
    }

    let mut new_source = Source::new(
        &args.initial_version,
        &args.artifact_url_template,
        args.update_scheme,
    )
    .with_unpack(args.unpack)
    .with_pinned(args.pinned)
    .with_git_url(args.git_repo_url)
    .with_tag_prefix(args.tag_prefix);

    new_source.pinned = args.pinned;

    if let Some(hash) = args.force_hash {
        new_source.hash = hash;
    } else {
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
    }

    sources.inner.insert(args.source_name.clone(), new_source);

    if let Err(e) = sources.write_to_file(source_file_path) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        info!("added new source {}", args.source_name);
        ExitCode::SUCCESS
    }
}
