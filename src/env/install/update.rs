use std::{
    collections::HashMap,
    env::{current_exe, temp_dir},
    fs::create_dir_all,
    process::Command,
};

use crate::{
    core::{Error, Result, config},
    env::install::{run_command, see_command, update_mode::UpdateMode},
};

#[derive(Debug, Default)]
struct Listing<'a> {
    head: Option<&'a str>,
    latest_tag: Option<(&'a str, &'a str)>,
    tags: Vec<(&'a str, &'a str)>,
    branches: HashMap<&'a str, &'a str>,
    other: Vec<(&'a str, &'a str)>,
}

pub fn try_update(
    remote: &str,
    mode: &UpdateMode,
    manpages: bool,
) -> Result<bool> {
    let v = get_latest_version(remote, mode)?;
    if is_up_to_date(&v, mode) {
        return Ok(false);
    }

    println!(
        "Current version is {}. It will be updated to {}-{}",
        config::VERSION_STR,
        v.1.as_deref().unwrap_or_default(),
        v.0
    );

    update(remote, &v.0, manpages)?;
    Ok(true)
}

fn update(remote: &str, commit: &str, manapges: bool) -> Result<()> {
    let exe = current_exe()?;

    let dir = temp_dir().join("uamp-install");
    create_dir_all(&dir)?;
    let repo = dir.join("uamp");
    if !repo.exists() {
        run_command(
            Command::new("git")
                .current_dir(&dir)
                .args(["clone", remote, "uamp"]),
        )?;
    }

    run_command(
        Command::new("git")
            .current_dir(&repo)
            .args(["checkout", commit]),
    )?;
    run_command(
        Command::new("cargo")
            .current_dir(&repo)
            .env("UAMP_VERSION_COMMIT", commit)
            .args(["build", "-r"]),
    )?;

    println!("Using self install of the new uamp.");

    run_command(
        Command::new("target/release/uamp")
            .current_dir(&repo)
            .args([
                "internal",
                "install",
                "--man",
                &manapges.to_string(),
                "--exe",
            ])
            .arg(exe),
    )?;

    Ok(())
}

fn get_latest_version(
    remote: &str,
    mode: &UpdateMode,
) -> Result<(String, Option<String>)> {
    let list = git_ls(remote)?;
    let list = Listing::load(&list);
    match mode {
        UpdateMode::LatestTag => list
            .latest_tag
            .map(|(c, t)| (c.to_string(), Some(t.to_string())))
            .ok_or_else(|| Error::not_found().msg("No latest tag.")),
        UpdateMode::LatestCommit => list
            .head
            .map(|c| (c.to_string(), None))
            .ok_or_else(|| Error::not_found().msg("No HEAD on repository.")),
        UpdateMode::Branch(b) => list
            .branches
            .get(&b.as_str())
            .map(|c| (c.to_string(), None))
            .ok_or_else(|| {
                Error::not_found().msg(format!("Failed to find branch `{b}`"))
            }),
    }
}

fn is_up_to_date(
    latest: &(String, Option<String>),
    mode: &UpdateMode,
) -> bool {
    match mode {
        UpdateMode::LatestTag => {
            config::VERSION_NUMBER == &latest.1.as_ref().unwrap()[1..]
                && config::VERSION_COMMIT == Some(&latest.0)
        }
        _ => matches!(config::VERSION_COMMIT, Some(c) if c == latest.0),
    }
}

fn git_ls(remote: &str) -> Result<Vec<(String, String)>> {
    let listing = see_command(
        "git",
        [
            "-c",
            "version.suffix=-",
            "ls-remote",
            "--sort=v:refname",
            remote,
        ],
    )?;
    let mut res = vec![];
    for itm in listing.split('\n') {
        let Some((commit, site)) = itm.split_once('\t') else {
            return Error::invalid_value()
                .msg("Failed to list git remote.")
                .reason("Malformed git output.")
                .err();
        };
        res.push((commit.trim().to_string(), site.trim().to_string()));
    }
    Ok(res)
}

impl<'a> Listing<'a> {
    pub fn load(list: &'a [(String, String)]) -> Self {
        let mut res = Self::default();

        for (c, r) in list {
            if r == "HEAD" && res.head.is_none() {
                res.head = Some(c);
            } else if let Some(tag) = r.strip_prefix("refs/tags/") {
                res.tags.push((c, tag));
            } else if let Some(branch) = r.strip_prefix("refs/heads/") {
                res.branches.insert(branch, c);
            } else {
                res.other.push((c, r));
            }
        }

        res.latest_tag = res.tags.last().copied();
        res
    }
}
