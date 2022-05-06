mod key;
mod schema;

use anyhow::{anyhow, Context, Result};
use proc_macro_error::{emit_call_site_error, emit_call_site_warning, proc_macro_error};
use quote::quote;
use syn::{AttributeArgs, ItemStruct, Lit, Meta, NestedMeta};

use std::{fs::File, io::BufReader};

use crate::schema::{Schema, SchemaList};

fn parse_schema(args: &AttributeArgs) -> Result<Schema> {
    let mut schema_path = None;
    let mut schema_id = None;

    for nested_meta in args.iter() {
        if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
            if name_value.path.is_ident("file") {
                if let Lit::Str(ref lit_str) = name_value.lit {
                    schema_path.replace(lit_str.value());
                } else {
                    emit_call_site_error!("expected a string literal after `file = `");
                }
            } else if name_value.path.is_ident("id") {
                if let Lit::Str(ref lit_str) = name_value.lit {
                    schema_id.replace(lit_str.value());
                } else {
                    emit_call_site_error!("expected a string literal after `id = `");
                }
            }
        }
    }

    let schema_path = schema_path.ok_or_else(|| anyhow!("expected a file meta"))?;
    let schema_id = schema_id.ok_or_else(|| anyhow!("expected a file meta"))?;

    let file = File::open(&schema_path)
        .with_context(|| format!("failed to open file at {}", schema_path))?;

    let mut schema_list: SchemaList =
        quick_xml::de::from_reader(BufReader::new(file)).expect("failed to parse schema file");

    if schema_list.len() != 1 {
        emit_call_site_error!("schema file must have a single schema");
    }

    let mut schema = schema_list
        .pop()
        .ok_or_else(|| anyhow!("a schema from file"))?;

    schema.id = schema_id;

    Ok(schema)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn gen_settings(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = syn::parse_macro_input!(attr as AttributeArgs);
    let item = syn::parse_macro_input!(item as ItemStruct);

    if !item.fields.is_empty() {
        emit_call_site_warning!("any struct field would be ignored")
    }

    let schema = parse_schema(&attr).expect("failed to parse schema");
    let schema_id = schema.id;

    let ident = item.ident;

    let mut keys_token_stream = proc_macro2::TokenStream::new();

    for key in &schema.keys {
        keys_token_stream.extend(key.to_token_stream())
    }

    let expanded = quote! {
        #[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct #ident(gio::Settings);

        impl #ident {
            pub const NONE: Option<&'static gio::Settings> = None;

            pub fn new() -> Self {
                Self(gio::Settings::new(#schema_id))
            }

            #keys_token_stream
        }

        impl std::ops::Deref for #ident {
            type Target = gio::Settings;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for #ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl Default for #ident {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
