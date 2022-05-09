use proc_macro_error::{abort, abort_call_site, emit_call_site_error, emit_error};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    AttributeArgs, Lit, Meta, NestedMeta, Token,
};

use std::{collections::HashMap, fs::File, io::BufReader};

use super::{
    generators::{GetResult, KeyGenerators, Override},
    schema::{KeySignature as SchemaKeySignature, SchemaList},
};

fn parse_name_value<'a, 'b>(
    haystack: impl IntoIterator<Item = &'b NestedMeta>,
    needles: &[&'a str],
    err_msg: &str,
) -> HashMap<&'a str, String> {
    let mut ret = HashMap::new();

    for nested_meta in haystack {
        if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
            for needle in needles {
                if name_value.path.is_ident(needle) {
                    if let Lit::Str(ref lit_str) = name_value.lit {
                        ret.insert(*needle, lit_str.value());
                    } else {
                        emit_error!(
                            name_value.span(),
                            "expected a string literal after `{} = `",
                            needle
                        );
                    }
                } else if name_value
                    .path
                    .get_ident()
                    .map_or(false, |ident| !needles.contains(&&*ident.to_string()))
                {
                    emit_error!(name_value.span(), err_msg);
                }
            }
        } else {
            emit_error!(nested_meta.span(), err_msg);
        }
    }

    ret
}

fn parse_schema_path_and_id(args: &AttributeArgs) -> (String, Option<String>) {
    let mut found = parse_name_value(
        args,
        &["file", "id"],
        "expected `#[gen_settings(file = \"path/to/schema\")]` or `#[gen_settings(file = \"path/to/schema\", id = \"org.some.id\")]`"
    );

    let schema_path = found
        .remove("file")
        .unwrap_or_else(|| abort!(args.first().unwrap().span(), "expected a `file` attribute"));

    let schema_id = found.remove("id");

    (schema_path, schema_id)
}

fn parse_schema_list(args: &AttributeArgs) -> (SchemaList, Option<String>) {
    let (schema_path, schema_id) = parse_schema_path_and_id(args);

    let span = args.first().unwrap().span();

    let file = File::open(&schema_path)
        .unwrap_or_else(|err| abort!(span, "failed to open file at {}: {:?}", schema_path, err));

    let schema_list: SchemaList =
        quick_xml::de::from_reader(BufReader::new(file)).unwrap_or_else(|err| {
            abort!(
                span,
                "failed to parse schema file at {}: {:?}",
                schema_path,
                err
            )
        });

    (schema_list, schema_id)
}

/// Returns (HashMap, HashMap);
/// the former containing signature to override;
/// the latter containing key name to override.
fn parse_overrides(
    known_signatures: &[SchemaKeySignature],
    known_key_names: &[&str],
    attrs: &[syn::Attribute],
) -> (
    HashMap<SchemaKeySignature, Override>,
    HashMap<String, Override>,
) {
    let mut signature_overrides = HashMap::new();
    let mut key_name_overrides = HashMap::new();

    for attr in attrs {
        if attr.path.is_ident("gen_settings_define") {
            let meta = attr
                .parse_meta()
                .unwrap_or_else(|err| abort!(attr.span(), "failed to parse meta: {:?}", err));

            if let Meta::List(ref meta_list) = meta {
                let mut found = parse_name_value(
                    &meta_list.nested,
                    &["key_name", "signature", "arg_type", "ret_type"],
                    "expected `#[gen_settings_define(signature | key_name = \"signature | key-name\", arg_type = \"arg_type\", ret_type = \"ret_type\")]`",
                );

                let signature = found.remove("signature");
                let key_name = found.remove("key_name");

                let arg_type = found
                    .remove("arg_type")
                    .unwrap_or_else(|| abort!(meta_list.span(), "expected a `arg_type` attribute"));
                let ret_type = found
                    .remove("ret_type")
                    .unwrap_or_else(|| abort!(meta_list.span(), "expected a `ret_type` attribute"));

                match (signature, key_name) {
                    (Some(signature), None) => {
                        let signature = SchemaKeySignature::Type(signature);

                        if !known_signatures.contains(&signature) {
                            emit_error!(
                                meta_list.span(),
                                "useless define for signature `{}`",
                                signature
                            );
                        }

                        if signature_overrides.get(&signature).is_some() {
                            emit_error!(
                                meta_list.span(),
                                "duplicate override for signature `{}`",
                                signature
                            );
                        }

                        signature_overrides
                            .insert(signature, Override::Define { arg_type, ret_type });
                    }
                    (None, Some(key_name)) => {
                        if !known_key_names.contains(&&*key_name) {
                            emit_error!(meta_list.span(), "key_name `{}` not found", key_name);
                        }

                        if key_name_overrides.get(&key_name).is_some() {
                            emit_error!(
                                meta_list.span(),
                                "duplicate override for key_name `{}`",
                                key_name
                            );
                        }

                        key_name_overrides
                            .insert(key_name, Override::Define { arg_type, ret_type });
                    }
                    (None, None) => {
                        emit_error!(
                            meta_list.span(),
                            "must have either `signature` or `key_name` attribute"
                        );
                    }
                    (Some(_), Some(_)) => {
                        emit_error!(
                            meta_list.span(),
                            "must only have either a `signature` or a `key_name` attribute"
                        );
                    }
                }
            }
        } else if attr.path.is_ident("gen_settings_skip") {
            let meta = attr
                .parse_meta()
                .unwrap_or_else(|err| abort!(attr.span(), "failed to parse meta: {:?}", err));

            if let Meta::List(ref meta_list) = meta {
                let mut found = parse_name_value(
                    &meta_list.nested,
                    &["key_name", "signature"],
                    "expected `#[gen_settings_skip(signature | key_name = \"signature | key-name\")]`",
                );

                let signature = found.remove("signature");
                let key_name = found.remove("key_name");

                match (signature, key_name) {
                    (Some(signature), None) => {
                        let signature = SchemaKeySignature::Type(signature);

                        if !known_signatures.contains(&signature) {
                            emit_error!(attr.span(), "useless skip for signature `{}`", signature);
                        }

                        if signature_overrides.get(&signature).is_some() {
                            emit_error!(
                                attr.span(),
                                "duplicate override for signature `{}`",
                                signature
                            );
                        }

                        signature_overrides.insert(signature, Override::Skip);
                    }
                    (None, Some(key_name)) => {
                        if !known_key_names.contains(&&*key_name) {
                            emit_error!(meta_list.span(), "key_name `{}` not found", key_name);
                        }

                        if key_name_overrides.get(&key_name).is_some() {
                            emit_error!(
                                meta_list.span(),
                                "duplicate override for key_name `{}`",
                                key_name
                            );
                        }

                        key_name_overrides.insert(key_name, Override::Skip);
                    }
                    (None, None) => {
                        emit_error!(
                            meta_list.span(),
                            "must have either `signature` or `key_name` attribute"
                        );
                    }
                    (Some(_), Some(_)) => {
                        emit_error!(
                            meta_list.span(),
                            "must only have either a `signature` or a `key_name` attribute"
                        );
                    }
                }
            }
        } else {
            emit_error!(
                attr.span(),
                "expected `#[gen_settings_define( .. )]` or `#[gen_settings_skip( .. )]`"
            );
        }
    }

    (signature_overrides, key_name_overrides)
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

pub fn impl_gen_settings(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = syn::parse_macro_input!(attr as AttributeArgs);
    let settings_struct = syn::parse_macro_input!(item as SettingsStruct);

    let (schema_list, schema_id) = parse_schema_list(&attr);
    let mut schemas = schema_list.schemas;

    if schemas.len() != 1 {
        emit_call_site_error!("schema file must have a single schema");
    }

    let schema = schemas
        .pop()
        .unwrap_or_else(|| abort_call_site!("expected a schema from file"));

    let known_signatures = schema
        .keys
        .iter()
        .map(|key| key.signature())
        .collect::<Vec<_>>();
    let known_key_names = schema
        .keys
        .iter()
        .map(|key| key.name.as_str())
        .collect::<Vec<_>>();
    let (signature_overrides, key_name_overrides) =
        parse_overrides(&known_signatures, &known_key_names, &settings_struct.attrs);
    let enums = schema_list
        .enums
        .iter()
        .map(|enum_| (enum_.id.to_string(), enum_.clone()))
        .collect::<HashMap<_, _>>();
    let flags = schema_list
        .flags
        .iter()
        .map(|flag| (flag.id.to_string(), flag.clone()))
        .collect::<HashMap<_, _>>();
    let mut key_generators = KeyGenerators::with_defaults(enums, flags);
    key_generators.add_signature_overrides(signature_overrides);
    key_generators.add_key_name_overrides(key_name_overrides);

    let mut aux_token_stream = proc_macro2::TokenStream::new();
    let mut keys_token_stream = proc_macro2::TokenStream::new();

    for key in &schema.keys {
        match key_generators.get(key) {
            GetResult::Skip => (),
            GetResult::Some(generator) => {
                keys_token_stream.extend(generator.to_token_stream());

                if let Some(aux) = generator.auxiliary() {
                    aux_token_stream.extend(aux);
                }
            }
            GetResult::Unknown => {
                emit_call_site_error!(
                    "unsupported {} signature used by key `{}`; consider using `#[gen_settings_define( .. )]` or skip it with `#[gen_settings_skip( .. )]`",
                    &key.signature(),
                    &key.name,
                )
            }
        }
    }

    let constructor_token_stream = if let Some(ref schema_id) = schema_id {
        quote! {
            pub fn new() -> Self {
                Self(gio::Settings::new(#schema_id))
            }
        }
    } else {
        quote! {
            pub fn new(schema_id: &str) -> Self {
                Self(gio::Settings::new(schema_id))
            }
        }
    };

    let struct_ident = &settings_struct.ident;

    let mut expanded = quote! {
        #aux_token_stream

        #[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #settings_struct

        impl #struct_ident {
            #constructor_token_stream

            #keys_token_stream
        }

        impl std::ops::Deref for #struct_ident {
            type Target = gio::Settings;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for #struct_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl std::fmt::Debug for #struct_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }
    };

    if schema_id.is_some() {
        expanded.extend(quote! {
            impl Default for #struct_ident {
                fn default() -> Self {
                    Self::new()
                }
            }
        });
    }

    proc_macro::TokenStream::from(expanded)
}
