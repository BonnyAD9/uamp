use std::io;

use pareg::Pareg;

use crate::core::{self, Error, Result, config::default_config_path};

use super::help::help_config;

#[derive(Debug, Clone)]
pub enum ConfigAction {
    EditFile,
    PrintPath,
    PrintDefault,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub actions: Vec<ConfigAction>,
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
                "--" => break,
                _ => args.err_unknown_argument().err()?,
            }
        }

        if self.actions.is_empty() {
            self.actions.push(ConfigAction::EditFile);
        }

        Ok(())
    }

    pub fn act(&self) -> Result<()> {
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
            }
        }

        Ok(())
    }
}
