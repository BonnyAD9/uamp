use std::{ffi::OsStr, fs, path::Path, process::Command};

use crate::core::{ErrCtx, Error, Result};

mod update;
mod update_mode;

pub use update::*;
pub use update_mode::*;

#[cfg(feature = "no-self-update")]
pub const ALLOW_SELF_UPDATE: bool = false;
#[cfg(not(feature = "no-self-update"))]
pub const ALLOW_SELF_UPDATE: bool = true;

pub const DEFAULT_REMOTE: &str = "https://github.com/BonnyAD9/uamp.git";

#[cfg(unix)]
pub const INSTALL_MANPAGES: bool = true;
#[cfg(not(unix))]
pub const INSTALL_MANPAGES: bool = false;

pub fn install(root: &Path, exe: &Path, man: bool) -> Result<()> {
    let exe = root.join(exe.strip_prefix("/").unwrap_or(exe));
    if exe.exists() {
        println!("Uamp is already installed, using tmp file for move.");
        let tmpexe = exe.join(".tmp");
        if tmpexe.exists() {
            return Error::Unexpected(
                ErrCtx::new("Tmp path {tmpexe:?} already exists.").into(),
            )
            .err();
        }
        fs::copy("target/release/uamp", &tmpexe)?;
        fs::rename(tmpexe, exe)?;
    } else {
        println!("Uamp is not installed, moving directly.");
        fs::copy("target/release/uamp", exe)?;
    }

    #[cfg(unix)]
    if man {
        println!("Installing manpages.");
        let man1 = Path::new("other/manpages/uamp.1.gz");
        let man5 = Path::new("other/manpages/uamp.5.gz");

        if !man1.exists() {
            run_command(
                Command::new("gzip").args(["-k", "other/manpages/uamp.1"]),
            )?;
        }
        if !man5.exists() {
            run_command(
                Command::new("gzip").args(["-k", "other/manpages/uamp.5"]),
            )?;
        }

        fs::copy(man1, "/usr/share/man/man1/uamp.1.gz")?;
        fs::copy(man5, "/usr/share/man/man1/uamp.5.gz")?;
    }

    Ok(())
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
