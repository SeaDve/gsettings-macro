use serde::Deserialize;

use crate::key::Key;

#[derive(Deserialize)]
pub(crate) struct Root {
    pub schemalist: SchemaList,
}

#[derive(Deserialize)]
pub(crate) struct SchemaList {
    #[serde(rename = "schema")]
    schemas: Vec<Schema>,
}

impl SchemaList {
    pub fn into_vec(self) -> Vec<Schema> {
        self.schemas
    }
}

#[derive(Deserialize)]
pub(crate) struct Schema {
    #[serde(rename = "key")]
    pub keys: Vec<Box<dyn Key>>,
}
