use serde::Deserialize;

use proc_macro_error::abort_call_site;

#[derive(Debug, Deserialize)]
pub struct SchemaList {
    #[serde(rename = "enum", default)]
    pub enums: Vec<Enum>,
    #[serde(default)]
    pub flags: Vec<Flag>,
    #[serde(rename = "schema")]
    pub schemas: Vec<Schema>,
}

#[derive(Debug, Deserialize)]
pub struct Enum {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "value")]
    pub values: Vec<EnumValues>,
}

#[derive(Debug, Deserialize)]
pub struct EnumValues {
    #[serde(rename = "@nick")]
    pub nick: String,
    #[serde(rename = "@value")]
    pub value: i32,
}

#[derive(Debug, Deserialize)]
pub struct Flag {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "value")]
    pub values: Vec<FlagValues>,
}

#[derive(Debug, Deserialize)]
pub struct FlagValues {
    #[serde(rename = "@nick")]
    pub nick: String,
    #[serde(rename = "@value")]
    pub value: u32,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    #[serde(rename = "key")]
    pub keys: Vec<Key>,
    #[serde(rename = "@id")]
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Key {
    #[serde(rename = "@type")]
    type_: Option<String>,
    #[serde(rename = "@enum")]
    enum_id: Option<String>,
    #[serde(rename = "@flags")]
    flag_id: Option<String>,
    #[serde(rename = "@name")]
    pub name: String,
    pub default: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub choices: Option<Choices>,
    pub range: Option<Range>,
}

pub enum KeySignature {
    Type(String),
    Enum(String),
    Flag(String),
}

impl PartialEq for KeySignature {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Type(l0), Self::Type(r0)) => l0 == r0,
            (Self::Enum(l0), Self::Enum(r0)) => l0 == r0,
            (Self::Flag(l0), Self::Flag(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for KeySignature {}

impl std::hash::Hash for KeySignature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            KeySignature::Type(type_) => type_.hash(state),
            KeySignature::Enum(enum_) => enum_.hash(state),
            KeySignature::Flag(flag) => flag.hash(state),
        }
    }
}

impl std::fmt::Display for KeySignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeySignature::Type(type_) => write!(f, "`{}` type", type_),
            KeySignature::Enum(enum_) => write!(f, "`{}` enum", enum_),
            KeySignature::Flag(flag) => write!(f, "`{}` flag", flag),
        }
    }
}

impl Key {
    pub fn signature(&self) -> KeySignature {
        match (&self.type_, &self.enum_id, &self.flag_id) {
            (Some(type_name), None, None) => KeySignature::Type(type_name.to_string()),
            (None, Some(enum_id), None) => KeySignature::Enum(enum_id.to_string()),
            (None, None, Some(flag_id)) => KeySignature::Flag(flag_id.to_string()),
            _ => abort_call_site!(
                "expected one of `type`, `enum` or `flags` specified attribute on key in the schema"
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Choices {
    #[serde(rename = "choice")]
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Range {
    #[serde(rename = "@max")]
    pub max: Option<String>,
    #[serde(rename = "@min")]
    pub min: Option<String>,
}
