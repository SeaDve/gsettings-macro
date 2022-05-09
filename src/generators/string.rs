use heck::ToPascalCase;
use quote::quote;
use syn::{spanned::Spanned, Ident};

use super::{Context, KeyGenerator, SchemaKey};
use crate::schema::Choice as KeyChoice;

pub fn key_generator(key: &SchemaKey) -> KeyGenerator<'_> {
    if let Some(ref choices) = key.choices {
        let choice_type_name = key.name.to_pascal_case();
        let choice_enum = choice_enum(&choice_type_name, &choices.choices);
        KeyGenerator::new(key, Context::new_with_aux(&choice_type_name, choice_enum))
    } else {
        KeyGenerator::new(key, Context::new_dissimilar("&str", "String"))
    }
}

fn choice_enum(name: &str, choices: &[KeyChoice]) -> proc_macro2::TokenStream {
    let variant_names = choices
        .iter()
        .map(|choice| choice.value.as_str())
        .collect::<Vec<_>>();

    let variant_idents = variant_names
        .iter()
        .map(|variant| Ident::new(&variant.to_pascal_case(), variant.span()))
        .collect::<Vec<_>>();

    let from_variant_arms =
        variant_names
            .iter()
            .zip(variant_idents.iter())
            .map(|(variant_name, variant_ident)| {
                quote! {
                    #variant_name => Some(Self::#variant_ident)
                }
            });

    let to_variant_arms =
        variant_names
            .iter()
            .zip(variant_idents.iter())
            .map(|(variant_name, variant_ident)| {
                quote! {
                    Self::#variant_ident => gio::glib::ToVariant::to_variant(#variant_name)
                }
            });

    let ident = Ident::new(name, name.span());

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum #ident {
            #(#variant_idents),*
        }

        impl gio::glib::StaticVariantType for #ident {
            fn static_variant_type() -> std::borrow::Cow<'static, gio::glib::VariantTy> {
                std::borrow::Cow::Borrowed(gio::glib::VariantTy::STRING)
            }
        }

        impl gio::glib::FromVariant for #ident {
            fn from_variant(variant: &gio::glib::Variant) -> Option<Self> {
                match variant.get::<String>()?.as_str() {
                    #(#from_variant_arms),*,
                    _ => None,
                }
            }
        }

        impl gio::glib::ToVariant for #ident {
            fn to_variant(&self) -> gio::glib::Variant {
                match self {
                    #(#to_variant_arms),*
                }
            }
        }
    }
}
