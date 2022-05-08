mod string;

use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::Ident;

use crate::schema::Key as SchemaKey;

pub enum Override {
    Define { arg_type: String, ret_type: String },
    Skip,
}

pub enum GetResult<'a> {
    Some(KeyGenerator<'a>),
    Skip,
    Unknown,
}

pub struct KeyGenerators {
    signatures: HashMap<String, Context>,
    key_names: HashMap<String, Context>,
    signature_skips: HashSet<String>,
    key_name_skips: HashSet<String>,
}

impl KeyGenerators {
    pub fn with_defaults() -> Self {
        let mut this = Self {
            signatures: HashMap::new(),
            key_names: HashMap::new(),
            signature_skips: HashSet::new(),
            key_name_skips: HashSet::new(),
        };

        // Defaults: Basic types that could easily be implemented automatically
        this.insert_signature("b", Context::new_auto("bool"));
        this.insert_signature("i", Context::new_auto("i32"));
        this.insert_signature("u", Context::new_auto("u32"));
        this.insert_signature("x", Context::new_auto("i64"));
        this.insert_signature("t", Context::new_auto("u64"));
        this.insert_signature("d", Context::new_auto("f64"));
        this.insert_signature("(ii)", Context::new_auto("(i32, i32)"));
        this.insert_signature("as", Context::new_auto_dissimilar("&[&str]", "Vec<String>"));

        this
    }

    /// Add contexts that has higher priority than default, but lower than
    /// key_name overrides
    pub fn add_signature_overrides(&mut self, overrides: HashMap<String, Override>) {
        for (signature, item) in overrides {
            match item {
                Override::Define { arg_type, ret_type } => {
                    self.insert_signature(
                        &signature,
                        Context::new_auto_dissimilar(&arg_type, &ret_type),
                    );
                }
                Override::Skip => {
                    self.signature_skips.insert(signature);
                }
            }
        }
    }

    /// Add contexts that has higher priority than both default and signature overrides.
    pub fn add_key_name_overrides(&mut self, overrides: HashMap<String, Override>) {
        for (key_name, item) in overrides {
            match item {
                Override::Define { arg_type, ret_type } => {
                    self.key_names
                        .insert(key_name, Context::new_auto_dissimilar(&arg_type, &ret_type));
                }
                Override::Skip => {
                    self.key_name_skips.insert(key_name);
                }
            }
        }
    }

    pub fn get<'a>(&'a self, key: &'a SchemaKey) -> GetResult<'a> {
        if self.key_name_skips.contains(&key.name) {
            return GetResult::Skip;
        }

        if self.signature_skips.contains(&key.type_) {
            return GetResult::Skip;
        }

        if let Some(context) = self.key_names.get(&key.name) {
            return GetResult::Some(KeyGenerator::new(key, context.clone()));
        }

        // Auto types
        if let Some(context) = self.signatures.get(&key.type_) {
            return GetResult::Some(KeyGenerator::new(key, context.clone()));
        }

        // Manual types
        match key.type_.as_str() {
            "s" => GetResult::Some(string::key_generator(key)),
            _ => GetResult::Unknown,
        }
    }

    fn insert_signature(&mut self, signature: &str, context: Context) {
        self.signatures.insert(signature.into(), context);
    }
}

pub struct KeyGenerator<'a> {
    key: &'a SchemaKey,
    context: Context,
}

impl<'a> KeyGenerator<'a> {
    fn new(key: &'a SchemaKey, context: Context) -> Self {
        Self { key, context }
    }

    fn docs(&self) -> String {
        match &self.context {
            Context::Auto { .. } => {
                let mut doc_buf = String::new();
                if let Some(ref summary) = self.key.summary {
                    if !summary.is_empty() {
                        doc_buf.push_str(summary);
                        doc_buf.push('\n');
                    }
                }
                if let Some(ref default) = self.key.default {
                    if !default.is_empty() {
                        doc_buf.push('\n');
                        doc_buf.push_str(&format!("default: {}", default));
                    }
                }
                // only needed for numerical types
                if let Some(ref range) = self.key.range {
                    let min_is_some = range.min.as_ref().map_or(false, |min| !min.is_empty());
                    let max_is_some = range.max.as_ref().map_or(false, |max| !max.is_empty());

                    if min_is_some || max_is_some {
                        doc_buf.push('\n');
                        doc_buf.push('\n');
                    }
                    if min_is_some {
                        doc_buf.push_str(&format!("min: {}", range.min.as_ref().unwrap()));
                    }
                    if min_is_some && max_is_some {
                        doc_buf.push(';');
                        doc_buf.push(' ');
                    }
                    if max_is_some {
                        doc_buf.push_str(&format!("max: {}", range.max.as_ref().unwrap()));
                    }
                }
                doc_buf
            }
            Context::Manual { doc, .. } => doc.clone(),
        }
    }

    pub fn aux(&self) -> Option<proc_macro2::TokenStream> {
        if let Context::Manual { ref auxiliary, .. } = self.context {
            return auxiliary.clone();
        }

        None
    }
}

impl quote::ToTokens for KeyGenerator<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let docs = self.docs();
        let key_name = self.key.name.as_str();
        let key_name_snake_case = key_name.to_snake_case();
        let getter_func_ident = Ident::new(&key_name_snake_case, Span::call_site());

        match &self.context {
            Context::Auto { arg_type, ret_type } => {
                let setter_func_ident = format_ident!("set_{}", getter_func_ident);
                let try_setter_func_ident = format_ident!("try_set_{}", getter_func_ident);

                let get_type = syn::parse_str::<syn::Type>(ret_type)
                    .unwrap_or_else(|_| panic!("Invalid type `{}`", ret_type));
                let set_type = syn::parse_str::<syn::Type>(arg_type)
                    .unwrap_or_else(|_| panic!("Invalid type `{}`", arg_type));

                tokens.extend(quote! {
                    #[doc = #docs]
                    pub fn #setter_func_ident(&self, value: #set_type) {
                        self.#try_setter_func_ident(value).unwrap_or_else(|err| panic!("failed to set value for key `{}`: {:?}", #key_name, err))
                    }

                    #[doc = #docs]
                    pub fn #try_setter_func_ident(&self, value: #set_type) -> std::result::Result<(), gio::glib::BoolError> {
                        gio::prelude::SettingsExtManual::set(&self.0, #key_name, &value)
                    }

                    #[doc = #docs]
                    pub fn #getter_func_ident(&self) -> #get_type {
                        gio::prelude::SettingsExtManual::get(&self.0, #key_name)
                    }
                });
            }
            Context::Manual { functions, .. } => {
                tokens.extend(functions.clone());
            }
        }

        let connect_changed_func_ident = format_ident!("connect_{}_changed", getter_func_ident);
        let bind_func_ident = format_ident!("bind_{}", getter_func_ident);
        let create_action_func_ident = format_ident!("create_{}_action", getter_func_ident);

        // Common items that even `Context::Manual` should not implement manually
        tokens.extend(quote! {
            #[doc = #docs]
            pub fn #connect_changed_func_ident(&self, f: impl Fn(&gio::Settings) + 'static) -> gio::glib::SignalHandlerId {
                gio::prelude::SettingsExt::connect_changed(&self.0, Some(#key_name), move |settings, _| {
                    f(settings)
                })
            }

            #[doc = #docs]
            pub fn #bind_func_ident<'a>(&'a self, object: &'a impl gio::glib::object::IsA<gio::glib::Object>, property: &'a str) -> gio::BindingBuilder<'a> {
                gio::prelude::SettingsExtManual::bind(&self.0, #key_name, object, property)
            }

            #[doc = #docs]
            pub fn #create_action_func_ident(&self) -> gio::Action {
                gio::prelude::SettingsExt::create_action(&self.0, #key_name)
            }
        })
    }
}

#[derive(Clone)]
pub enum Context {
    Auto {
        arg_type: String,
        ret_type: String,
    },
    Manual {
        functions: proc_macro2::TokenStream,
        auxiliary: Option<proc_macro2::TokenStream>,
        doc: String,
    },
}

impl Context {
    pub fn new_auto(type_: &str) -> Self {
        Self::new_auto_dissimilar(type_, type_)
    }

    pub fn new_auto_dissimilar(arg_type: &str, ret_type: &str) -> Self {
        Self::Auto {
            arg_type: arg_type.to_string(),
            ret_type: ret_type.to_string(),
        }
    }

    pub fn new_manual_with_aux(
        functions: proc_macro2::TokenStream,
        auxiliary: proc_macro2::TokenStream,
        doc: String,
    ) -> Self {
        Self::Manual {
            functions,
            auxiliary: Some(auxiliary),
            doc,
        }
    }
}
