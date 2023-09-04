use std::str::FromStr;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use itertools::Itertools;

use super::{
    code::{get_code_string, string_to_code},
    modifier::{get_modifier_string, string_to_modifier},
    err::Error,
};

/// Represents hotkey
#[derive(Hash, PartialEq, Clone)]
pub struct Hotkey {
    code: Code,
    modifiers: Modifiers,
}

impl Eq for Hotkey {}

impl Hotkey {
    /// Creates [`HotKey`] from this hotkey
    pub fn as_hot_key(&self) -> HotKey {
        HotKey::new(Some(self.modifiers), self.code)
    }
}

impl ToString for Hotkey {
    fn to_string(&self) -> String {
        get_modifier_string(&self.modifiers)
            + "+"
            + get_code_string(&self.code)
    }
}

impl FromStr for Hotkey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let mut ms = Modifiers::empty();
        let mut cs = None;

        for s in s
            .chars()
            .filter(|c| !c.is_whitespace()) // remove whitespace
            .map(|c| if c == '-' { '_' } else { c }) // map '-' to '_'
            .flat_map(|c| c.to_lowercase()) // convert to lower case
            .group_by(|c| *c == '+') // split by '+'
            .into_iter()
            .filter(|(b, _)| !b) // remove the  '+'
            .map(|(_, i)| i.collect::<String>())
        {
            if let Some(m) = string_to_modifier(&s) {
                ms |= m;
                continue;
            }
            if let Some(c) = string_to_code(&s) {
                if cs.is_some() {
                    return Err(Error::MultipleKeys);
                }
                cs = Some(c)
            } else {
                return Err(Error::UnknownKey(s));
            }
        }

        Ok(Hotkey {
            code: cs.ok_or(Error::NoKey)?,
            modifiers: ms,
        })
    }
}
