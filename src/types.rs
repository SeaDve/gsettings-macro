use std::ops::Deref;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SchemaList {
    #[serde(rename = "$value")]
    schemas: Vec<Schema>,
}

impl Deref for SchemaList {
    type Target = Vec<Schema>;

    fn deref(&self) -> &Self::Target {
        &self.schemas
    }
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub id: String,
    #[serde(rename = "$value")]
    pub keys: Vec<Key>,
}

#[derive(Debug, Deserialize)]
pub struct Key {
    pub name: String,
    #[serde(rename = "type", default)]
    pub type_: String,
}
