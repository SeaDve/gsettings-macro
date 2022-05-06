mod basic;
mod string;

use quote::ToTokens;

#[typetag::serde(tag = "type")]
pub(crate) trait Key: ToTokens {
    fn aux(&self) -> Vec<proc_macro2::TokenStream> {
        vec![]
    }
}
