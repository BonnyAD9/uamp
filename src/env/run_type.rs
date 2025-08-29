use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, Serialize, Deserialize, Default, PartialEq, Eq,
)]
pub enum RunType {
    #[default]
    Background,
    WebClient,
}
