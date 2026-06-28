use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::{
    config::AutoTag,
    library::{Tag, TagId},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tags(pub(super) HashMap<TagId, Tag>);

impl Tags {
    pub fn init_tag(&mut self, tag: &AutoTag) {
        self.0
            .entry(tag.name.clone())
            .or_insert_with(|| tag.clone().into());
    }

    pub fn init_tags<'a>(
        &mut self,
        tags: impl IntoIterator<Item = &'a AutoTag>,
    ) {
        for t in tags {
            self.init_tag(t);
        }
    }
}
