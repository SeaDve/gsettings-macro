mod key;
mod schema;

use anyhow::{anyhow, Context, Result};
use proc_macro_error::{emit_call_site_error, emit_call_site_warning, proc_macro_error};
use quote::quote;
use syn::{AttributeArgs, ItemStruct, Lit, Meta, NestedMeta};

use std::{fs::File, io::Read};

use crate::schema::{Root as SchemaFileRoot, Schema};

fn parse_schema_path_and_id(args: &AttributeArgs) -> Result<(String, Option<String>)> {
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
            } else {
                emit_call_site_error!("expected `#[gen_settings(file = \"path/to/schema\")]` or `#[gen_settings(file = \"path/to/schema\", id = \"org.some.id\")]`")
            }
        }
    }

    let schema_path = schema_path.ok_or_else(|| anyhow!("expected a file meta"))?;

    Ok((schema_path, schema_id))
}

fn parse_schema(args: &AttributeArgs) -> Result<(Schema, Option<String>)> {
    let (schema_path, schema_id) = parse_schema_path_and_id(args)?;

    use quickxml_to_serde::{Config, JsonArray, JsonType, NullValue};

    let mut xml_content = String::new();
    File::open(&schema_path)
        .with_context(|| format!("failed to open file at {}", schema_path))?
        .read_to_string(&mut xml_content)?;

    let config = Config::new_with_custom_values(true, "", "#text", NullValue::Ignore)
        .add_json_type_override(
            "schemalist/schema",
            JsonArray::Always(JsonType::AlwaysString),
        )
        .add_json_type_override(
            "schemalist/schema/key",
            JsonArray::Always(JsonType::AlwaysString),
        )
        .add_json_type_override(
            "schemalist/schema/key/default",
            JsonArray::Infer(JsonType::AlwaysString),
        )
        .add_json_type_override(
            "schemalist/schema/key/summary",
            JsonArray::Infer(JsonType::AlwaysString),
        )
        .add_json_type_override(
            "schemalist/schema/key/description",
            JsonArray::Infer(JsonType::AlwaysString),
        )
        .add_json_type_override(
            "schemalist/schema/key/choices",
            JsonArray::Infer(JsonType::AlwaysString),
        )
        .add_json_type_override(
            "schemalist/schema/key/choices/choice",
            JsonArray::Always(JsonType::AlwaysString),
        );

    // ideally we would use xml directly, but it is harder to deal with than json
    let json_value = quickxml_to_serde::xml_string_to_json(xml_content, &config)?;

    let root: SchemaFileRoot =
        serde_json::from_value(json_value).expect("failed to parse schema file");

    let mut schema_list = root.schemalist.into_vec();

    if schema_list.len() != 1 {
        emit_call_site_error!("schema file must have a single schema");
    }

    let schema = schema_list
        .pop()
        .ok_or_else(|| anyhow!("a schema from file"))?;

    Ok((schema, schema_id))
}

/// Needs `gio` in scope.
///
/// Not specifying the id in the attribute will require the id in the [`new`] constructor.
/// Additionally, it will not implement [`Default`].
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

    let (schema, schema_id) = parse_schema(&attr).expect("failed to parse schema");

    let ident = item.ident;

    let mut aux_token_stream = proc_macro2::TokenStream::new();
    let mut keys_token_stream = proc_macro2::TokenStream::new();

    for key in &schema.keys {
        for token_stream in key.aux() {
            aux_token_stream.extend(token_stream);
        }

        keys_token_stream.extend(key.to_token_stream())
    }

    let constructor_token_stream = if let Some(ref id) = schema_id {
        quote! {
            pub fn new() -> Self {
                Self(gio::Settings::new(#id))
            }
        }
    } else {
        quote! {
            pub fn new(schema_id: &str) -> Self {
                Self(gio::Settings::new(schema_id))
            }
        }
    };

    let mut expanded = quote! {
        #aux_token_stream

        #[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct #ident(gio::Settings);

        impl #ident {
            #constructor_token_stream

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

        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }
    };

    if schema_id.is_some() {
        expanded.extend(quote! {
            impl Default for #ident {
                fn default() -> Self {
                    Self::new()
                }
            }
        });
    }

    proc_macro::TokenStream::from(expanded)
}
