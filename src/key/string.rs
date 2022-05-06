use anyhow::Result;
use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Serialize};
use syn::{spanned::Spanned, Ident};

#[derive(Serialize, Deserialize)]
pub struct Choice {
    value: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Choices {
    #[serde(rename = "choice")]
    choices: Vec<Choice>,
}

impl Choices {
    fn to_enum_token_stream(&self, ident: &Ident) -> Result<proc_macro2::TokenStream> {
        let variants = self
            .choices
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

        Ok(quote! {
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
        })
    }
}

#[derive(Serialize, Deserialize)]
struct StringKey {
    name: String,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    choices: Choices,
}

impl StringKey {
    fn get_docs(&self) -> String {
        let mut buf = String::new();

        if let Some(ref summary) = self.summary {
            if !summary.is_empty() {
                buf.push_str(summary);
                buf.push('\n');
            }
        }

        if let Some(ref default) = self.default {
            let display = if self.has_choice() {
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

    fn has_choice(&self) -> bool {
        !self.choices.choices.is_empty()
    }

    fn enum_ident(&self) -> Ident {
        let name = self.name.to_pascal_case();
        Ident::new(&name, name.span())
    }
}

#[typetag::serde(name = "s")]
impl crate::key::Key for StringKey {
    fn aux(&self) -> Vec<proc_macro2::TokenStream> {
        let mut aux = Vec::new();

        if self.has_choice() {
            aux.push(
                self.choices
                    .to_enum_token_stream(&self.enum_ident())
                    .expect("Failed to convert choice to token stream"),
            );
        }

        aux
    }
}

impl ToTokens for StringKey {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let key_name = self.name.as_str();
        let key_name_snake_case = key_name.to_snake_case();

        let getter_func_name = Ident::new(&key_name_snake_case, Span::call_site());
        let setter_func_name = format_ident!("set_{}", getter_func_name);

        let str_slice_type = syn::parse_str::<syn::Type>("&str").unwrap();
        let gstring_type = syn::parse_str::<syn::Type>("gio::glib::GString").unwrap();

        let choice_type_ident = self.enum_ident();

        let docs = self.get_docs();

        let setter_func = if self.has_choice() {
            quote! {
                #[doc = #docs]
                pub fn #setter_func_name(&self, value: #choice_type_ident) -> std::result::Result<(), gio::glib::BoolError> {
                    self.0.set_string(#key_name, value.to_string().as_str())
                }
            }
        } else {
            quote! {
                #[doc = #docs]
                pub fn #setter_func_name(&self, value: #str_slice_type) -> std::result::Result<(), gio::glib::BoolError> {
                    self.0.set_string(#key_name, value)
                }
            }
        };

        let getter_func = if self.has_choice() {
            quote! {
                #[doc = #docs]
                pub fn #getter_func_name(&self) -> #choice_type_ident {
                    #choice_type_ident::from_str(&self.0.string(#key_name))
                }
            }
        } else {
            quote! {
                #[doc = #docs]
                pub fn #getter_func_name(&self) -> #gstring_type {
                    self.0.string(#key_name)
                }
            }
        };

        tokens.extend(setter_func);
        tokens.extend(getter_func);
    }
}
