use std::{env, path::Path};

use pareg::Pareg;

use crate::core::Result;

use super::help::help_shell;

#[derive(Debug, Clone)]
pub enum ShellAction {
    TabComplete,
}

#[derive(Debug, Default)]
pub struct Shell {
    pub actions: Vec<ShellAction>,
    pub script: bool,
}

impl Shell {
    pub(super) fn parse(
        &mut self,
        args: &mut Pareg,
        color: bool,
    ) -> Result<()> {
        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => help_shell(color),
                "-s" | "--script" => self.script = true,
                "tab" | "tab-completion" => {
                    self.actions.push(ShellAction::TabComplete)
                }
                "--" => break,
                _ => args.err_unknown_argument().err()?,
            }
        }

        Ok(())
    }

    pub fn act(&self) {
        for a in &self.actions {
            match a {
                ShellAction::TabComplete => self.tab_complete(),
            }
        }
    }

    fn tab_complete(&self) {
        const SCRIPT: &str = include_str!("../../scripts/tab-complete.sh");

        let uamp_path = env::args()
            .next()
            .map(|p| {
                Path::new(&p)
                    .canonicalize()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or(p)
            })
            .unwrap_or_else(|| "uamp".to_owned());

        if self.script {
            println!("__uamp_path='{uamp_path}'");
            print!("{SCRIPT}");
        } else {
            print!("eval __uamp_path='{uamp_path}'; {SCRIPT}");
        }
    }
}
