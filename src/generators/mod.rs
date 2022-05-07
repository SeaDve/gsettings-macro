mod string;

use heck::ToSnakeCase;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::Ident;

use crate::schema::Key as SchemaKey;

pub struct KeyGenerators {
    inner: HashMap<String, Context>,
}

impl KeyGenerators {
    pub fn get<'a>(&'a self, key: &'a SchemaKey) -> Option<KeyGenerator<'a>> {
        // Auto types
        if let Some(context) = self.inner.get(&key.type_) {
            return Some(KeyGenerator::new(key, context.clone()));
        }

        // Manual types
        match key.type_.as_str() {
            "s" => Some(string::key_generator(key)),
            _ => None,
        }
    }

    pub fn insert(&mut self, signature: &str, context: Context) {
        self.inner.insert(signature.into(), context);
    }
}

impl Default for KeyGenerators {
    fn default() -> Self {
        let mut this = Self {
            inner: HashMap::new(),
        };
        // Known basic types
        this.insert("b", Context::new_auto("bool"));
        this.insert("i", Context::new_auto("i32"));
        this.insert("u", Context::new_auto("u32"));
        this.insert("x", Context::new_auto("i64"));
        this.insert("t", Context::new_auto("u64"));
        this.insert("d", Context::new_auto("f64"));
        this.insert("(ii)", Context::new_auto("(i32, i32)"));
        this.insert("as", Context::new_auto_dissimilar("&[&str]", "Vec<String>"));
        this
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
        let getter_func_name = Ident::new(&key_name_snake_case, Span::call_site());

        match &self.context {
            Context::Auto { arg_type, ret_type } => {
                let setter_func_name = format_ident!("set_{}", getter_func_name);
                let try_setter_func_name = format_ident!("try_set_{}", getter_func_name);

                let get_type = syn::parse_str::<syn::Type>(ret_type)
                    .unwrap_or_else(|_| panic!("Invalid type {}", ret_type));
                let set_type = syn::parse_str::<syn::Type>(arg_type)
                    .unwrap_or_else(|_| panic!("Invalid type {}", arg_type));

                tokens.extend(quote! {
                    #[doc = #docs]
                    pub fn #setter_func_name(&self, value: #set_type) {
                        self.#try_setter_func_name(value).unwrap_or_else(|err| panic!("failed to set value for key `{}`: {:?}", #key_name, err))
                    }

                    #[doc = #docs]
                    pub fn #try_setter_func_name(&self, value: #set_type) -> std::result::Result<(), gio::glib::BoolError> {
                        gio::prelude::SettingsExtManual::set(&self.0, #key_name, &value)
                    }

                    #[doc = #docs]
                    pub fn #getter_func_name(&self) -> #get_type {
                        gio::prelude::SettingsExtManual::get(&self.0, #key_name)
                    }
                });
            }
            Context::Manual { functions, .. } => {
                tokens.extend(functions.clone());
            }
        }

        let connect_changed_name = format_ident!("connect_{}_changed", getter_func_name);
        let bind_name = format_ident!("bind_{}", getter_func_name);
        let create_action_name = format_ident!("create_{}_action", getter_func_name);

        // Common items that even `Context::Manual` should not implement manually
        tokens.extend(quote! {
            #[doc = #docs]
            pub fn #connect_changed_name(&self, f: impl Fn(&gio::Settings) + 'static) -> gio::glib::SignalHandlerId {
                gio::prelude::SettingsExt::connect_changed(&self.0, Some(#key_name), move |settings, _| {
                    f(settings)
                })
            }

            #[doc = #docs]
            pub fn #bind_name<'a>(&'a self, object: &'a impl gio::glib::object::IsA<gio::glib::Object>, property: &'a str) -> gio::BindingBuilder<'a> {
                gio::prelude::SettingsExtManual::bind(&self.0, #key_name, object, property)
            }

            #[doc = #docs]
            pub fn #create_action_name(&self) -> gio::Action {
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
