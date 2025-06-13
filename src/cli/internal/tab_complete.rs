use std::{borrow::Cow, fs::read_dir, path::Path};

use pareg::Pareg;

type CowStr = Cow<'static, str>;

use crate::core::{Result, config::Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMode {
    Basic,
    Instance,
    Run,
    Config,
    Shell,
    Update,
    Internal,
    InternalTabComplete,
    Help,
    Invalid,
    Port,
    Color,
    Print,
    UpdateRemote,
    UpdateMode,
    None,
}

#[derive(Debug)]
pub struct TabComplete {
    cur: String,
    mode: TabMode,
}

impl TabComplete {
    pub fn new(args: &mut Pareg) -> Result<Self> {
        let idx = args.next_arg::<usize>()?.saturating_sub(1);

        let mut i = 0;
        let mut mode2 = TabMode::Basic;
        let mut mode = TabMode::Basic;
        let mut cur = String::new();
        args.next();

        while let Some(arg) = args.next() {
            if i == idx {
                cur = arg.into();
                args.skip_all();
                break;
            }

            if matches!(mode, TabMode::Port | TabMode::Color | TabMode::None) {
                mode = mode2;
                mode2 = TabMode::Basic;
            }

            match (mode, arg) {
                (TabMode::Basic, "i" | "instance") => mode = TabMode::Instance,
                (TabMode::Basic, "h" | "help") => mode = TabMode::Help,
                (TabMode::Basic, "run") => mode = TabMode::Run,
                (TabMode::Basic, "cfg" | "conf" | "config") => {
                    mode = TabMode::Config
                }
                (TabMode::Basic, "sh" | "shell") => mode = TabMode::Shell,
                (TabMode::Basic, "update") => mode = TabMode::Update,
                (TabMode::Basic, "internal") => mode = TabMode::Internal,
                (
                    TabMode::Basic | TabMode::Instance | TabMode::Run,
                    "-p" | "--port",
                ) => {
                    mode2 = mode;
                    mode = TabMode::Port;
                }
                (
                    TabMode::Basic | TabMode::Instance | TabMode::Run,
                    "-a" | "--address",
                ) => {
                    mode2 = mode;
                    mode = TabMode::None;
                }
                (TabMode::Basic, "--color") => {
                    mode2 = mode;
                    mode = TabMode::Color;
                }
                (TabMode::Basic, "--print") => {
                    mode2 = mode;
                    mode = TabMode::Print;
                }
                (TabMode::Update, "--remote") => {
                    mode2 = mode;
                    mode = TabMode::UpdateRemote;
                }
                (TabMode::Update, "-m" | "--mode") => {
                    mode2 = mode;
                    mode = TabMode::UpdateMode;
                }
                (TabMode::Internal, "tab-complete") => {
                    mode = TabMode::InternalTabComplete
                }
                (TabMode::Internal, _) => mode = TabMode::Invalid,
                (TabMode::InternalTabComplete, _) => mode = TabMode::Basic,
                (_, "--") => mode = TabMode::Basic,
                _ => {}
            }

            i += 1;
        }

        Ok(Self { cur, mode })
    }

    pub fn act(&self, conf: &Config) -> Result<()> {
        let p = &|a| println!("{a}");

        match self.mode {
            TabMode::Basic => basic_args(conf, &self.cur, p),
            TabMode::Instance => instance_args(conf, &self.cur, p),
            TabMode::Run => run_args(conf, &self.cur, p),
            TabMode::Config => config_args(conf, &self.cur, p),
            TabMode::Shell => shell_args(conf, &self.cur, p),
            TabMode::Update => update_args(conf, &self.cur, p),
            TabMode::Internal => internal_args(conf, &self.cur, p),
            TabMode::InternalTabComplete => {}
            TabMode::Help => help_args(conf, &self.cur, p),
            TabMode::Invalid => {}
            TabMode::Port => port_args(conf, &self.cur, p),
            TabMode::Color => color_args(conf, &self.cur, p),
            TabMode::Print => print_args(conf, &self.cur, p),
            TabMode::UpdateRemote => {}
            TabMode::UpdateMode => update_mode_args(conf, &self.cur, p),
            TabMode::None => {}
        }

        Ok(())
    }
}

macro_rules! prefixed_map {
    ($arg:ident, $p:ident, $conf:ident, $def:expr, $($pf:literal => $($f:ident)? $([$c:ident])?),* $(,)?) => {
        $(if let Some(s) = $arg.strip_prefix($pf) {
            $($f($conf, s, &|a| $p(($pf.to_owned() + &a).into()));)?
            $(select_args($c, s, $p);)?
        } else)* {
            $def
        }
    };
}

fn basic_args(conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    prefixed_map!(arg, p, conf, select_args(BASIC_ARG, arg, p),
        "--color=" => color_args,
        "--colour=" => color_args,
        "-I" => instance_args,
        "-R" => run_args,
        "-C" => config_args,
        "-H" => help_args,
    );
}

fn instance_args(conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    prefixed_map!(arg, p, conf, {
            select_args(INSTANCE_ARG, arg, p);
            select_args(ANY_CONTROL_MSG, arg, p);
        },
        "pp=" => [PLAY_PAUSE_ARG],
        "play-pause=" => [PLAY_PAUSE_ARG],
        "mute=" => [MUTE_ARG],
        "p=" => file_args,
        "play=" => file_args,
        "al=" => alias_args,
        "alias=" => alias_args,
        "restart=" => file_args,
    );
}

fn run_args(conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    prefixed_map!(arg, p, conf, {
            select_args(RUN_ARG, arg, p);
            select_args(ANY_CONTROL_MSG, arg, p);
        },
        "pp=" => [PLAY_PAUSE_ARG],
        "play-pause=" => [PLAY_PAUSE_ARG],
        "mute=" => [MUTE_ARG],
        "al=" => alias_args,
        "alias=" => alias_args,
        "restart=" => file_args,
    );
}

fn config_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(CONFIG_ARG, arg, p);
}

fn shell_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(SHELL_ARG, arg, p);
}

fn update_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(UPDATE_ARG, arg, p);
}

fn internal_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(INTERANAL_ARG, arg, p);
}

fn help_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(HELP_ARG, arg, p);
}

fn port_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(PORT_ARG, arg, p);
}

fn color_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(COLOR_ARG, arg, p);
}

fn print_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(PRINT_ARG, arg, p);
}

fn update_mode_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    select_args(UPDATE_MODE_ARG, arg, p);
}

fn file_args(_conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    _ = file_args_inner(arg, p);
}

fn file_args_inner(arg: &str, p: &impl Fn(CowStr)) -> Result<()> {
    let mut prefix = "";
    let path: &Path = if arg.ends_with('/') {
        Path::new(arg)
    } else {
        let p = Path::new(arg).parent();
        if p.is_none() || p.unwrap() == Path::new("") {
            prefix = "./";
            Path::new("./")
        } else {
            #[allow(clippy::unnecessary_unwrap)]
            p.unwrap()
        }
    };

    let hide = arg.is_empty() || arg.ends_with('/');

    for f in read_dir(path)? {
        let f = f?;
        let s = f.path();
        if hide && f.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let s = s.to_string_lossy();
        let s = s.strip_prefix(prefix).unwrap_or(&s);

        if s.starts_with(arg) {
            let mut s = s.to_string();
            if f.file_type()?.is_dir() {
                s += "/";
            }
            p(s.into())
        }
    }

    Ok(())
}

fn alias_args(conf: &Config, arg: &str, p: &impl Fn(CowStr)) {
    for (k, v) in conf.control_aliases() {
        if k.starts_with(arg) {
            if v.args().is_empty() {
                p(k.to_string().into())
            } else {
                p(format!("{k}{{{}}}", v.args().join(",")).into())
            }
        }
    }
}

fn select_args(
    sel: &'static [&'static [&'static str]],
    arg: &str,
    p: impl Fn(CowStr),
) {
    let i = sel
        .iter()
        .copied()
        .filter_map(move |a| {
            a.iter().rev().copied().max_by_key(|a| {
                if *a == arg {
                    u32::MAX
                } else if a.starts_with(arg) {
                    1
                } else {
                    0
                }
            })
        })
        .map(|a| a.into());

    for i in i {
        p(i)
    }
}

const BASIC_ARG: &[&[&str]] = &[
    &["instance", "i"],
    &["help", "h"],
    &["run"],
    &["conf", "config", "cfg"],
    &["sh", "shell"],
    &["update"],
    &["internal"],
    &["-h", "-?", "--help"],
    &["--version"],
    &["-p", "--port"],
    &["-a", "--address"],
    &["--color", "--colour"],
    &["--"],
    &["-I"],
    &["-R"],
    &["-C"],
    &["-H"],
    &["-v", "--verbose"],
];

const INSTANCE_ARG: &[&[&str]] = &[
    &["info", "nfo"],
    &["show"],
    &["play", "p"],
    &["list", "query", "l"],
    &["-h", "-?", "--help"],
    &["-p", "--port"],
    &["-a", "--address"],
    &["-v", "--verbose"],
    &["--"],
    // ANY_CONTROL_MSG
];

const HELP_ARG: &[&[&str]] = &[
    &["basic"],
    &["instance", "i"],
    &["run"],
    &["conf", "config", "cfg"],
    &["sh", "shell"],
    &["update"],
    &["internal"],
    &["help", "h", "-h", "-?", "--help"],
    &["all", "elp"],
    &["control-msg", "control-message", "cmsg"],
    &["format", "formats"],
    &["port"],
    &["query"],
    &["filter"],
    &["order"],
    &["--"],
];

const RUN_ARG: &[&[&str]] = &[
    &["-h", "-?", "--help"],
    &["-d", "--detach"],
    &["-p", "--port"],
    &["-a", "--address"],
    // ANY_CONTROL_MSG
];

const CONFIG_ARG: &[&[&str]] = &[
    &["-h", "-?", "--help"],
    &["-e", "--edit", "--edit-file"],
    &["-p", "--print-path"],
    &["--default"],
    &["-v", "--verbose"],
    &["--"],
];

const SHELL_ARG: &[&[&str]] = &[
    &["-h", "-?", "--help"],
    &["-s", "--script"],
    &["tab", "tab-completion"],
    &["--"],
];

const UPDATE_ARG: &[&[&str]] = &[
    &["-h", "-?", "--help"],
    &["-f", "--force"],
    &["--remote"],
    &["--man"],
    &["--no-man"],
    &["-m", "--mode"],
];

const INTERANAL_ARG: &[&[&str]] =
    &[&["-h", "-?", "--help"], &["tab-complete"]];

const ANY_CONTROL_MSG: &[&[&str]] = &[
    &["play-pause", "pp"],
    &["volume-up", "vol-up", "vu"],
    &["volume-down", "vol-down", "vd"],
    &["next-song", "ns"],
    &["previous-song", "ps"],
    &["playlist-jump", "pj"],
    &["vol=", "volume=", "v="],
    &["mute"],
    &["load-songs"],
    &["shuffle-playlist", "shuffle"],
    &["exit", "close", "x"],
    &["seek=", "seek-to="],
    &["fast-forward", "ff"],
    &["rewind", "rw"],
    &["sort-playlist=", "sort="],
    &["pop", "pop-playlist"],
    &["playlist-add-policy", "add-policy", "pap"],
    &["save"],
    &["alias=", "al="],
    &["playlist-end-action", "playlist-end", "pl-end", "spea"],
    &["set-playlist", "sp"],
    &["push", "push-playlist"],
    &["push-cur", "push-with-cur", "pc"],
    &["queue", "q"],
    &["play-next", "queue-next", "qn"],
    &["restart"],
    &["rps=", "reorder-playlist-stack="],
];

const PORT_ARG: &[&[&str]] =
    &[&["default", "-"], &["debug"], &["release", "uamp"]];

const COLOR_ARG: &[&[&str]] = &[&["auto"], &["always"], &["never"]];

const PRINT_ARG: &[&[&str]] = &[&["pretty"], &["debug"], &["json"]];

const UPDATE_MODE_ARG: &[&[&str]] = &[
    &["tag", "latest-tag", "LatestTag"],
    &["commit", "latest-commit", "LatestCommit"],
];

const PLAY_PAUSE_ARG: &[&[&str]] = &[&["play"], &["pause"]];

const MUTE_ARG: &[&[&str]] = &[&["true"], &["false"]];
