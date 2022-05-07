use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Ident};

use super::{ContextItem, GenerationItem, SchemaKey};
use crate::schema::Choice as KeyChoice;

pub fn generation_item(key: &SchemaKey) -> GenerationItem<'_> {
    if let Some(ref choices) = key.choices {
        let choice_type_name = key.name.to_pascal_case();
        let choice_type_ident = Ident::new(&choice_type_name, choice_type_name.span());

        let key_name = key.name.as_str();
        let key_name_snake_case = key_name.to_snake_case();

        let getter_func_name = Ident::new(&key_name_snake_case, Span::call_site());
        let setter_func_name = format_ident!("set_{}", getter_func_name);
        let try_setter_func_name = format_ident!("try_set_{}", getter_func_name);

        let docs = docs(key);

        let func = quote! {
            #[doc = #docs]
            pub fn #setter_func_name(&self, value: #choice_type_ident) {
                self.#try_setter_func_name(value).unwrap_or_else(|err| panic!("failed to set value for key `{}`: {:?}", #key_name, err))
            }

            #[doc = #docs]
            pub fn #try_setter_func_name(&self, value: #choice_type_ident) -> std::result::Result<(), gio::glib::BoolError> {
                gio::prelude::SettingsExt::set_string(&self.0, #key_name, value.to_string().as_str())
            }

            #[doc = #docs]
            pub fn #getter_func_name(&self) -> #choice_type_ident {
                #choice_type_ident::from_str(&gio::prelude::SettingsExt::string(&self.0, #key_name))
            }
        };

        let choice_enum = choice_enum(&choice_type_ident, &choices.choices);

        GenerationItem::new(
            key,
            ContextItem::new_complex_with_aux(func, choice_enum, docs),
        )
    } else {
        GenerationItem::new(key, ContextItem::new_basic_dissimilar("&str", "String"))
    }
}

fn choice_enum(ident: &Ident, choices: &[KeyChoice]) -> proc_macro2::TokenStream {
    let variants = choices
        .iter()
        .map(|choice| choice.value.as_str())
        .collect::<Vec<_>>();

    let variant_idents = variants
        .iter()
        .map(|variant| Ident::new(&variant.to_pascal_case(), variant.span()))
        .collect::<Vec<_>>();

    let from_str_arms = variants
        .iter()
        .zip(variant_idents.iter())
        .map(|(variant_name, variant_ident)| {
            quote! {
                #variant_name => Self::#variant_ident
            }
        })
        .collect::<Vec<_>>();

    let display_arms = variants
        .iter()
        .zip(variant_idents.iter())
        .map(|(variant_name, variant_ident)| {
            quote! {
                Self::#variant_ident =>  write!(f, #variant_name)
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum #ident {
            #(#variant_idents),*
        }

        impl #ident {
            pub fn from_str(string: &str) -> Self {
                match string {
                    #(#from_str_arms),*,
                    other => panic!("Invalid variant `{}`", other),
                }
            }
        }

        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self {
                    #(#display_arms),*
                }
            }
        }
    }
}

fn is_complex(key: &SchemaKey) -> bool {
    if let Some(ref choices) = key.choices {
        return !choices.choices.is_empty();
    }

    false
}

fn docs(key: &SchemaKey) -> String {
    let mut buf = String::new();

    if let Some(ref summary) = key.summary {
        if !summary.is_empty() {
            buf.push_str(summary);
            buf.push('\n');
        }
    }

    if let Some(ref default) = key.default {
        let display = if is_complex(key) {
            default.to_pascal_case()
        } else {
            default.to_string()
        };

        if !default.is_empty() {
            buf.push('\n');
            buf.push_str(&format!("default: {}", display));
        }
    }

    buf
}
