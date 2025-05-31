use std::io;

use itertools::Itertools;
use pareg::{Pareg, parse_arg};

use crate::core::{self, Error, Result, config::default_config_path};

use super::{Props, help::help_config};

#[derive(Debug, Clone)]
pub enum ConfigAction {
    EditFile,
    PrintPath,
    PrintDefault,
    PrintAliases,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub actions: Vec<ConfigAction>,
    verbosity: Option<i32>,
}

impl Config {
    pub(super) fn parse(
        &mut self,
        args: &mut Pareg,
        color: bool,
    ) -> Result<()> {
        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => help_config(color),
                "-e" | "--edit" | "--edit-file" => {
                    self.actions.push(ConfigAction::EditFile);
                }
                "-p" | "--path" | "--print-path" => {
                    self.actions.push(ConfigAction::PrintPath);
                }
                "--default" => {
                    self.actions.push(ConfigAction::PrintDefault);
                }
                "--aliases" => self.actions.push(ConfigAction::PrintAliases),
                "--" => break,
                "-v" | "--verbose" => self.verbosity = Some(1),
                v if v.starts_with("-v") => {
                    self.verbosity =
                        Some(args.cur_manual(|a| parse_arg(&a[2..]))?);
                }
                _ => args.err_unknown_argument().err()?,
            }
        }

        if self.actions.is_empty() {
            self.actions.push(ConfigAction::EditFile);
        }

        Ok(())
    }

    pub fn act(
        &self,
        conf: &core::config::Config,
        props: &Props,
    ) -> Result<()> {
        for a in &self.actions {
            match a {
                ConfigAction::PrintPath => {
                    println!("{}", default_config_path().to_string_lossy())
                }
                ConfigAction::EditFile => {
                    edit::edit_file(default_config_path()).map_err(|e| {
                        Error::io(e)
                            .msg("Failed to open config file in editor.")
                    })?;
                }
                ConfigAction::PrintDefault => {
                    core::config::Config::default()
                        .to_json(io::stdout())
                        .map_err(|e| {
                            e.msg("Failed to print default configuration.")
                        })?;
                    println!();
                }
                ConfigAction::PrintAliases => {
                    if conf.control_aliases().is_empty() {
                        continue;
                    }
                    let verbosity = self.verbosity.unwrap_or(props.verbosity);
                    if verbosity >= 1 {
                        let mut al =
                            conf.control_aliases().iter().collect_vec();
                        al.sort_by_key(|(k, _)| *k);
                        let max_len = al
                            .iter()
                            .max_by_key(|(k, _)| k.len())
                            .unwrap()
                            .0
                            .len();
                        for (n, v) in al {
                            println!("{n:>max_len$} {v}");
                        }
                    } else {
                        for n in conf.control_aliases().keys().sorted() {
                            println!("{n}");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
