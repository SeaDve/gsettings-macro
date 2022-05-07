mod generators;
mod schema;

use anyhow::{anyhow, Context, Result};
use generators::Override;
use proc_macro_error::{abort, emit_call_site_error, emit_error, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    AttributeArgs, Lit, Meta, NestedMeta, Token,
};

use std::{collections::HashMap, fs::File, io::BufReader};

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

fn parse_struct_attributes(attrs: &[syn::Attribute]) -> Result<HashMap<String, Override>> {
    let mut overrides = HashMap::new();

    for attr in attrs {
        if attr.path.is_ident("gen_settings_define") {
            if let Meta::List(ref meta_list) = attr.parse_meta()? {
                let mut signature = None;
                let mut arg_type = None;
                let mut ret_type = None;

                for nested_meta in &meta_list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                        if name_value.path.is_ident("signature") {
                            if let Lit::Str(ref lit_str) = name_value.lit {
                                signature.replace(lit_str.value());
                            } else {
                                emit_error!(
                                    name_value.span(),
                                    "expected a string literal after `signature = `"
                                );
                            }
                        } else if name_value.path.is_ident("arg_type") {
                            if let Lit::Str(ref lit_str) = name_value.lit {
                                arg_type.replace(lit_str.value());
                            } else {
                                emit_error!(
                                    name_value.span(),
                                    "expected a string literal after `arg_type = `"
                                );
                            }
                        } else if name_value.path.is_ident("ret_type") {
                            if let Lit::Str(ref lit_str) = name_value.lit {
                                ret_type.replace(lit_str.value());
                            } else {
                                emit_error!(
                                    name_value.span(),
                                    "expected a string literal after `ret_type = `"
                                );
                            }
                        } else {
                            emit_call_site_error!(
                                "expected `signature`, `arg_type` and `ret_type`"
                            );
                        }
                    } else {
                        emit_call_site_error!("wrong meta");
                    }
                }

                overrides.insert(
                    signature
                        .take()
                        .unwrap_or_else(|| abort!(meta_list.span(), "expected `signature = \"\"`")),
                    Override::Define {
                        arg_type: arg_type.take().unwrap_or_else(|| {
                            abort!(meta_list.span(), "expected `arg_type = \"\"`")
                        }),
                        ret_type: ret_type.take().unwrap_or_else(|| {
                            abort!(meta_list.span(), "expected `ret_type = \"\"`")
                        }),
                    },
                );
            }
        } else if attr.path.is_ident("gen_settings_skip") {
            let signature: syn::LitStr = attr.parse_args()?;
            overrides.insert(signature.value(), Override::Skip);
        } else {
            emit_call_site_error!("expected `gen_settings_define` or `gen_settings_skip`");
        }
    }

    Ok(overrides)
}

struct SettingsStruct {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    struct_token: Token![struct],
    ident: syn::Ident,
    semi_token: Token![;],
}

impl Parse for SettingsStruct {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse()?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for SettingsStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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

    let mut key_generators = KeyGenerators::default();
    let overrides =
        parse_struct_attributes(&settings_struct.attrs).expect("failed to parse struct attributes");
    key_generators.add_overrides(overrides);

    for key in &schema.keys {
        match key_generators.get(key) {
            generators::GetResult::Skip => (),
            generators::GetResult::Some(generator) => {
                keys_token_stream.extend(generator.to_token_stream());

                if let Some(aux) = generator.aux() {
                    aux_token_stream.extend(aux);
                }
            }
            generators::GetResult::Unknown => {
                emit_call_site_error!(
                    "unsupported signature `{}` used by key `{}`; consider using #[gen_settings_define( .. )]",
                    &key.type_,
                    &key.name,
                )
            }
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
