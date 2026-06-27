use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::library::{Tag, TagId};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tags(pub(super) HashMap<TagId, Tag>);

impl Tags {}
