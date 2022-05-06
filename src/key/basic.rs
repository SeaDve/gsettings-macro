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
    ($name:ident, $arg_type:literal, $ret_type:literal, $gfunc_name:literal, $variant_type:literal) => {
        #[derive(Debug, Deserialize, Serialize)]
        struct $name {
            name: String,
        }

        #[typetag::serde(name = $variant_type)]
        impl crate::key::Key for $name {}

        impl ToTokens for $name {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                let key_name = self.name.as_str();
                let key_name_snake_case = key_name.to_snake_case();

                let getter_func_name = Ident::new(&key_name_snake_case, Span::call_site());
                let setter_func_name = format_ident!("set_{}", getter_func_name);

                let get_type = syn::parse_str::<syn::Type>($ret_type).unwrap_or_else(|_| panic!("Invalid type {}", $ret_type));
                let set_type = syn::parse_str::<syn::Type>($arg_type).unwrap_or_else(|_| panic!("Invalid type {}", $ret_type));

                let get_gfunc_name = Ident::new($gfunc_name, Span::call_site());
                let set_gfunc_name = format_ident!("set_{}", get_gfunc_name);

                tokens.extend(quote! {
                    pub fn #setter_func_name(&self, value: #set_type) -> std::result::Result<(), gio::glib::BoolError> {
                        self.0.#set_gfunc_name(#key_name, value)
                    }

                    pub fn #getter_func_name(&self) -> #get_type {
                        self.0.#get_gfunc_name(#key_name)
                    }
                });
            }
        }
    };
}

impl_basic_key!(BooleanKey, "bool", "bool", "boolean", "b");

impl_basic_key!(StringKey, "&str", "gio::glib::GString", "string", "s");
impl_basic_key!(
    StringVecKey,
    "&[&str]",
    "Vec<gio::glib::GString>",
    "strv",
    "as"
);

impl_basic_key!(IntKey, "i32", "i32", "int", "i");
impl_basic_key!(UIntKey, "u32", "u32", "uint", "u");

impl_basic_key!(Int64Key, "i64", "i64", "int64", "x");
impl_basic_key!(UInt64Key, "u64", "u64", "uint64", "t");

impl_basic_key!(DoubleKey, "f64", "f64", "double", "d");
