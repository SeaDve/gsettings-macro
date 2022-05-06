mod basic;

use quote::ToTokens;

#[typetag::serde(tag = "type")]
pub(crate) trait Key: ToTokens {}
