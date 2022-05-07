mod generators;
mod schema;

use anyhow::{anyhow, Context, Result};
use proc_macro_error::{emit_call_site_error, proc_macro_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    AttributeArgs, Lit, Meta, NestedMeta, Token,
};

use std::{fs::File, io::BufReader};

use self::{
    generators::KeyGenerators,
    schema::{Schema, SchemaList},
};

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

    let file = File::open(&schema_path)
        .with_context(|| format!("failed to open file at {}", schema_path))?;

    let schema_list: SchemaList = quick_xml::de::from_reader(BufReader::new(file))?;

    let mut schema_list = schema_list.into_vec();

    if schema_list.len() != 1 {
        emit_call_site_error!("schema file must have a single schema");
    }

    let schema = schema_list
        .pop()
        .ok_or_else(|| anyhow!("a schema from file"))?;

    Ok((schema, schema_id))
}

struct SettingsStruct {
    vis: syn::Visibility,
    attrs: Vec<syn::Attribute>,
    struct_token: Token![struct],
    ident: syn::Ident,
    semi_token: Token![;],
}

impl Parse for SettingsStruct {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            vis: input.parse()?,
            attrs: input.call(syn::Attribute::parse_outer)?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for SettingsStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(&self.attrs);
        self.vis.to_tokens(tokens);
        self.struct_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);

        let field: syn::FieldsUnnamed = syn::parse_quote!((gio::Settings));
        field.to_tokens(tokens);

        self.semi_token.to_tokens(tokens);
    }
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
    let settings_struct = syn::parse_macro_input!(item as SettingsStruct);

    let (schema, schema_id) = parse_schema(&attr).expect("failed to parse schema");

    let ident = &settings_struct.ident;

    let mut aux_token_stream = proc_macro2::TokenStream::new();
    let mut keys_token_stream = proc_macro2::TokenStream::new();

    let key_generators = KeyGenerators::default();

    for key in &schema.keys {
        if let Some(generator) = key_generators.get(key) {
            keys_token_stream.extend(generator.to_token_stream());

            if let Some(aux) = generator.aux() {
                aux_token_stream.extend(aux);
            }
        } else {
            emit_call_site_error!(
                "unsupported signature `{}` used by key `{}`",
                &key.type_,
                &key.name,
            )
        }
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
        #settings_struct

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
