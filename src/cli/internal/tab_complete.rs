use std::borrow::Cow;

use pareg::Pareg;

use crate::core::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMode {
    None,
    Instance,
    Run,
    Config,
    Shell,
    Internal,
    InternalTabComplete,
    Help,
    Invalid,
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
        let mut mode = TabMode::None;
        // TODO: Fix when updating pareg
        let mut cur = String::new();
        args.next();

        while let Some(arg) = args.next() {
            if i == idx {
                cur = arg.into();
                // TODO: Fix when updating pareg
                while args.next().is_some() {}
                break;
            }

            match (mode, arg) {
                (TabMode::None, "i" | "instance") => mode = TabMode::Instance,
                (TabMode::None, "h" | "help") => mode = TabMode::Help,
                (TabMode::None, "run") => mode = TabMode::Run,
                (TabMode::None, "cfg" | "conf" | "config") => {
                    mode = TabMode::Config
                }
                (TabMode::None, "sh" | "shell") => mode = TabMode::Shell,
                (TabMode::None, "internal") => mode = TabMode::Internal,
                (TabMode::Internal, "tab-complete") => {
                    mode = TabMode::InternalTabComplete
                }
                (TabMode::Internal, _) => mode = TabMode::Invalid,
                (TabMode::InternalTabComplete, _) => mode = TabMode::None,
                (_, "--") => mode = TabMode::None,
                _ => {}
            }

            i += 1;
        }

        Ok(Self { cur, mode })
    }

    pub fn act(&self) -> Result<()> {
        let mut args = vec![];

        match self.mode {
            TabMode::None => self.select_args(BASIC_ARG, &mut args),
            TabMode::Instance => {
                self.select_args(INSTANCE_ARG, &mut args);
                self.select_args(ANY_CONTROL_MSG, &mut args);
            }
            TabMode::Run => {
                self.select_args(RUN_ARG, &mut args);
                self.select_args(ANY_CONTROL_MSG, &mut args);
            }
            TabMode::Config => self.select_args(CONFIG_ARG, &mut args),
            TabMode::Shell => self.select_args(SHELL_ARG, &mut args),
            TabMode::Internal => self.select_args(INTERANAL_ARG, &mut args),
            TabMode::InternalTabComplete => {}
            TabMode::Help => self.select_args(HELP_ARG, &mut args),
            TabMode::Invalid => {}
        }

        for a in args {
            println!("{a}");
        }

        Ok(())
    }

    fn select_args(
        &self,
        sel: &'static [&'static [&'static str]],
        res: &mut Vec<Cow<'static, str>>,
    ) {
        res.extend(
            sel.iter()
                .copied()
                .filter_map(|a| {
                    a.iter().rev().copied().max_by_key(|a| {
                        if *a == self.cur {
                            u32::MAX
                        } else if a.starts_with(&self.cur) {
                            1
                        } else {
                            0
                        }
                    })
                })
                .map(|a| a.into()),
        );
    }
}

const BASIC_ARG: &[&[&str]] = &[
    &["instance", "i"],
    &["help", "h"],
    &["run"],
    &["conf", "config", "cfg"],
    &["sh", "shell"],
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
];

const INSTANCE_ARG: &[&[&str]] = &[
    &["info", "nfo"],
    &["play", "p"],
    &["list", "query", "l"],
    &["-h", "-?", "--help"],
    &["-p", "--port"],
    &["-a", "--address"],
    &["--"],
    // ANY_CONTROL_MSG
];

const HELP_ARG: &[&[&str]] = &[
    &["basic"],
    &["instance", "i"],
    &["run"],
    &["conf", "config", "cfg"],
    &["sh", "shell"],
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
    &["--"],
];

const SHELL_ARG: &[&[&str]] = &[
    &["-h", "-?", "--help"],
    &["-s", "--script"],
    &["tab", "tab-completion"],
    &["--"],
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
];
