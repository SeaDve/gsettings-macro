use serde::Deserialize;

use proc_macro_error::abort_call_site;

#[derive(Debug, Deserialize)]
pub struct SchemaList {
    #[serde(rename = "enum")]
    pub enums: Vec<Enum>,
    #[serde(rename = "schema")]
    pub schemas: Vec<Schema>,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(rename = "key")]
    pub keys: Vec<Key>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Enum {
    pub id: String,
    #[serde(rename = "value")]
    pub values: Vec<EnumValues>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnumValues {
    pub nick: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Key {
    #[serde(rename = "type")]
    type_: Option<String>,
    #[serde(rename = "enum")]
    enum_id: Option<String>,
    pub name: String,
    pub default: String,
    pub summary: Option<String>,
    pub choices: Option<Choices>,
    pub range: Option<Range>,
}

pub enum KeySignature {
    Type(String),
    Enum(String),
}

impl PartialEq for KeySignature {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Type(l0), Self::Type(r0)) => l0 == r0,
            (Self::Enum(l0), Self::Enum(r0)) => l0 == r0,
            (Self::Enum(_), Self::Type(_)) | (Self::Type(_), Self::Enum(_)) => false,
        }
    }
}

impl Eq for KeySignature {}

impl std::hash::Hash for KeySignature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            KeySignature::Type(type_) => type_.hash(state),
            KeySignature::Enum(enum_) => enum_.hash(state),
        }
    }
}

impl std::fmt::Display for KeySignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeySignature::Type(type_) => write!(f, "`{}` type", type_),
            KeySignature::Enum(enum_) => write!(f, "`{}` enum", enum_),
        }
    }
}

impl Key {
    pub fn signature(&self) -> KeySignature {
        match (&self.type_, &self.enum_id) {
            (None, Some(enum_id)) => KeySignature::Enum(enum_id.to_string()),
            (Some(type_name), None) => KeySignature::Type(type_name.to_string()),
            (None, None) => abort_call_site!("must have a type or enum"),
            (Some(_), Some(_)) => abort_call_site!("must either be a type or enum, not both"),
        }
    }
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

#[derive(Debug, Deserialize)]
pub struct Range {
    pub max: Option<String>,
    pub min: Option<String>,
}
