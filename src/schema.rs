use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Root {
    pub schemalist: SchemaList,
}

#[derive(Debug, Deserialize)]
pub struct SchemaList {
    #[serde(rename = "schema")]
    schemas: Vec<Schema>,
}

impl SchemaList {
    pub fn into_vec(self) -> Vec<Schema> {
        self.schemas
    }
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(rename = "key")]
    pub keys: Vec<Key>,
}

#[derive(Debug, Deserialize)]
pub struct Key {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub default: Option<String>,
    pub summary: Option<String>,
    pub choices: Option<Choices>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Choices {
    #[serde(rename = "choice")]
    pub choices: Vec<Choice>,
}
