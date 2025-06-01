mod mode;

use std::{
    collections::HashMap,
    env::{current_exe, temp_dir},
    ffi::OsStr,
    fs::{self, create_dir_all},
    path::Path,
    process::Command,
};

use crate::core::{ErrCtx, Error, Result, config};

pub use self::mode::*;

#[derive(Debug, Default)]
struct Listing<'a> {
    head: Option<&'a str>,
    latest_tag: Option<(&'a str, &'a str)>,
    tags: Vec<(&'a str, &'a str)>,
    branches: HashMap<&'a str, &'a str>,
    other: Vec<(&'a str, &'a str)>,
}

#[cfg(feature = "no-self-update")]
pub const ALLOW_SELF_UPDATE: bool = false;
#[cfg(not(feature = "no-self-update"))]
pub const ALLOW_SELF_UPDATE: bool = true;

pub const DEFAULT_REMOTE: &str = "https://github.com/BonnyAD9/uamp.git";

#[cfg(unix)]
pub const INSTALL_MANPAGES: bool = true;
#[cfg(not(unix))]
pub const INSTALL_MANPAGES: bool = false;

pub fn try_update(remote: &str, mode: &Mode, manpages: bool) -> Result<bool> {
    let v = get_latest_version(remote, mode)?;
    if is_up_to_date(&v, mode) {
        return Ok(false);
    }

    println!(
        "Current version is {}. It will be updated to {}-{}",
        config::VERSION_STR,
        v.1.as_ref().map(|a| a.as_str()).unwrap_or_default(),
        v.0
    );

    update(remote, &v.0, manpages)?;
    Ok(true)
}

fn update(remote: &str, commit: &str, manpages: bool) -> Result<()> {
    let exe = current_exe()?;
    let mut tmpexe = exe.as_os_str().to_owned();
    tmpexe.push(".tmp");
    let tmpexe_path = Path::new(&tmpexe);
    if tmpexe_path.exists() {
        return Error::Unexpected(
            ErrCtx::new("Tmp path {tmpexe:?} already exists.").into(),
        )
        .err();
    }

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

    fs::copy(repo.join("target/release/uamp"), tmpexe_path)?;
    fs::rename(tmpexe_path, exe)?;

    #[cfg(unix)]
    if manpages {
        let man1 = repo.join("other/manpages/uamp.1.gz");
        let man5 = repo.join("other/manpages/uamp.5.gz");

        if !man1.exists() {
            run_command(
                Command::new("gzip")
                    .current_dir(&repo)
                    .args(["-k", "other/manpages/uamp.1"]),
            )?;
        }
        if !man5.exists() {
            run_command(
                Command::new("gzip")
                    .current_dir(&repo)
                    .args(["-k", "other/manpages/uamp.5"]),
            )?;
        }

        fs::copy(man1, "/usr/share/man/man1/uamp.1.gz")?;
        fs::copy(man5, "/usr/share/man/man1/uamp.5.gz")?;
    }

    fs::remove_dir_all(dir)?;

    Ok(())
}

fn get_latest_version(
    remote: &str,
    mode: &Mode,
) -> Result<(String, Option<String>)> {
    let list = git_ls(remote)?;
    let list = Listing::load(&list);
    match mode {
        Mode::LatestTag => list
            .latest_tag
            .map(|(c, t)| (c.to_string(), Some(t.to_string())))
            .ok_or_else(|| {
                Error::NotFound(ErrCtx::new("No latest tag.").into())
            }),
        Mode::LatestCommit => {
            list.head.map(|c| (c.to_string(), None)).ok_or_else(|| {
                Error::NotFound(ErrCtx::new("No HEAD on repository.").into())
            })
        }
        Mode::Branch(b) => list
            .branches
            .get(&b.as_str())
            .map(|c| (c.to_string(), None))
            .ok_or_else(|| {
                Error::NotFound(ErrCtx::new("No such branch.").into())
                    .msg(format!("Failed to find branch `{b}`"))
            }),
    }
}

fn is_up_to_date(latest: &(String, Option<String>), mode: &Mode) -> bool {
    match mode {
        Mode::LatestTag => {
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
            return Error::NotFound(
                ErrCtx::new("Malformed git output.").into(),
            )
            .err();
        };
        res.push((commit.trim().to_string(), site.trim().to_string()));
    }
    Ok(res)
}

fn see_command<S: AsRef<OsStr>>(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = S>,
) -> Result<String> {
    let mut cmd = Command::new(cmd);
    cmd.args(args);

    let res = cmd.output()?;
    if res.status.success() {
        Ok(String::from_utf8_lossy(res.stdout.trim_ascii()).to_string())
    } else {
        Error::ChildFailed(
            ErrCtx::new(String::from_utf8_lossy(&res.stderr).to_string())
                .into(),
        )
        .msg(format!(
            "Command ({cmd:?}) failed wit exit code `{}`",
            res.status.code().unwrap_or(1)
        ))
        .err()
    }
}

fn run_command(cmd: &mut Command) -> Result<()> {
    println!("{cmd:?}");
    let res = cmd.spawn()?.wait()?;
    if res.success() {
        Ok(())
    } else {
        Error::ChildFailed(ErrCtx::new(String::new()).into())
            .msg(format!(
                "Command ({cmd:?}) failed wit exit code `{}`",
                res.code().unwrap_or(1)
            ))
            .err()
    }
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
