use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Serialize};
use syn::Ident;

/// Needs the following parameters:
/// - name: Name of Struct
/// - arg_type: Argument type used in setter (`&str`, `i64`, etc.)
/// - ret_type: Argument type used in getter (`glib::GString`, `i64`, etc.)
/// - call_name: What method to call in [`gio::Settings`] (`int`, `boolean`, etc.)
/// - variant_type: [`glib::Variant`] type string (`i`, `b`, etc.)
macro_rules! impl_basic_key {
    ($name:ident, $arg_type:literal, $ret_type:literal, $variant_type:literal) => {
        #[derive(Deserialize, Serialize)]
        struct $name {
            name: String,
            default: Option<String>,
            summary: Option<String>,
        }

        #[typetag::serde(name = $variant_type)]
        impl crate::key::Key for $name {}

        impl ToTokens for $name {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                let key_name = self.name.as_str();
                let key_name_snake_case = key_name.to_snake_case();

                let getter_func_name = Ident::new(&key_name_snake_case, Span::call_site());
                let setter_func_name = format_ident!("set_{}", getter_func_name);
                let try_setter_func_name = format_ident!("try_set_{}", getter_func_name);

                let get_type = syn::parse_str::<syn::Type>($ret_type).unwrap_or_else(|_| panic!("Invalid type {}", $ret_type));
                let set_type = syn::parse_str::<syn::Type>($arg_type).unwrap_or_else(|_| panic!("Invalid type {}", $ret_type));

                let mut doc_buf = String::new();

                if let Some(ref summary) = self.summary {
                    if !summary.is_empty() {
                        doc_buf.push_str(summary);
                        doc_buf.push('\n');
                    }
                }

                if let Some(ref default) = self.default {
                    if !default.is_empty() {
                        doc_buf.push('\n');
                        doc_buf.push_str(&format!("default: {}", default));
                    }
                }

                tokens.extend(quote! {
                    #[doc = #doc_buf]
                    pub fn #setter_func_name(&self, value: #set_type) {
                        self.#try_setter_func_name(value).unwrap_or_else(|err| panic!("failed to set value for key `{}`: {:?}", #key_name, err))
                    }

                    #[doc = #doc_buf]
                    pub fn #try_setter_func_name(&self, value: #set_type) -> std::result::Result<(), gio::glib::BoolError> {
                        gio::prelude::SettingsExtManual::set(&self.0, #key_name, &value)
                    }

                    #[doc = #doc_buf]
                    pub fn #getter_func_name(&self) -> #get_type {
                        gio::prelude::SettingsExtManual::get(&self.0, #key_name)
                    }
                });
            }
        }
    };
}

impl_basic_key!(BooleanKey, "bool", "bool", "b");

impl_basic_key!(StringVecKey, "&[&str]", "Vec<String>", "as");

impl_basic_key!(IntKey, "i32", "i32", "i");
impl_basic_key!(UIntKey, "u32", "u32", "u");

impl_basic_key!(Int64Key, "i64", "i64", "x");
impl_basic_key!(UInt64Key, "u64", "u64", "t");

impl_basic_key!(DoubleKey, "f64", "f64", "d");
