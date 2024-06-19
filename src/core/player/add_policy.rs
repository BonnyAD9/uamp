use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AddPolicy {
    End,
    Next,
    MixIn,
}
