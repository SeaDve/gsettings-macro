use std::ops::{Deref, DerefMut};

use serde::Deserialize;

use crate::key::Key;

#[derive(Deserialize)]
pub(crate) struct SchemaList {
    #[serde(rename = "$value")]
    schemas: Vec<Schema>,
}

impl Deref for SchemaList {
    type Target = Vec<Schema>;

    fn deref(&self) -> &Self::Target {
        &self.schemas
    }
}

impl DerefMut for SchemaList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.schemas
    }
}

#[derive(Deserialize)]
pub(crate) struct Schema {
    pub id: String,
    #[serde(rename = "$value")]
    pub keys: Vec<Box<dyn Key>>,
}
