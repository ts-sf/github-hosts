pub const GIT_REVISION: &str = {
    if let Some(revision) = option_env!("GIT_REVISION") {
        revision
    } else {
        git_version::git_version!(
            args = ["--always", "--abbrev=12", "--dirty", "--exclude", "*"],
            fallback = "no revision"
        )
    }
};

pub const VERSION: &str = const_str::concat!(env!("CARGO_PKG_VERSION"), "-", GIT_REVISION);
