use std::{
    ffi::OsStr,
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

use crate::core::{ErrCtx, Error, Result, config};

mod update;
mod update_mode;

use termal::printcln;
use tokio::runtime;
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

pub fn install(root: Option<&Path>, exe: &Path, man: bool) -> Result<()> {
    let exe = path_at_root(root, exe);
    if exe.exists() {
        println!("Uamp is already installed, using tmp file for move.");
        let tmpexe = exe.with_extension(".tmp");
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
        make_parent(&exe)?;
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

        let dst1 = path_at_root(root, "/usr/share/man/man1/uamp.1.gz");
        let dst5 = path_at_root(root, "/usr/share/man/man5/uamp.5.gz");
        make_parent(&dst1)?;
        make_parent(&dst5)?;

        fs::copy(man1, dst1)?;
        fs::copy(man5, dst5)?;

        println!("Installing icons.");
        let dsti = path_at_root(
            root,
            "/usr/share/icons/hicolor/256x256/apps/uamp.png",
        );
        let dsts = path_at_root(
            root,
            "/usr/share/icons/hicolor/scalable/apps/uamp.svg",
        );
        make_parent(&dsti)?;
        make_parent(&dsts)?;
        fs::copy("assets/img/icon_light256.png", dsti)?;
        fs::copy("assets/svg/icon_light.svg", dsts)?;

        println!("Installing desktop file.");
        let dstd = path_at_root(root, "/usr/share/applications/uamp.desktop");
        make_parent(&dstd)?;
        fs::copy("other/uamp.desktop", dstd)?;
    }

    println!("Creating client tar.");
    let client_dst = path_at_root(root, config::default_http_client_path());
    make_parent(&client_dst)?;
    run_single_tokio(tar_contents_of("src/client", client_dst))?;

    printcln!(
        "{'g}Success!{'_} uamp v{} is installed.",
        config::VERSION_STR
    );

    Ok(())
}

fn make_parent(path: impl AsRef<Path>) -> Result<()> {
    if let Some(p) = path.as_ref().parent() {
        create_dir_all(p)?;
    }
    Ok(())
}

fn path_at_root(
    root: Option<impl AsRef<Path>>,
    path: impl AsRef<Path>,
) -> PathBuf {
    let path = path.as_ref();
    root.map(|r| r.as_ref().join(path.strip_prefix("/").unwrap_or(path)))
        .unwrap_or_else(|| path.to_path_buf())
}

fn run_single_tokio<T>(task: impl Future<Output = Result<T>>) -> Result<T> {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(task)
}

async fn tar_contents_of(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<()> {
    let mut bld = tokio_tar::Builder::new(tokio::fs::File::create(dst).await?);
    bld.follow_symlinks(true);
    bld.append_dir_all("", src).await?;
    bld.finish().await?;
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
