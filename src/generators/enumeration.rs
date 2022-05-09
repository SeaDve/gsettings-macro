use heck::ToPascalCase;
use quote::quote;
use syn::{spanned::Spanned, Ident};

use super::{Context, KeyGenerator, SchemaEnum, SchemaKey};
use crate::schema::EnumValues as SchemaEnumValues;

pub fn key_generator<'a>(key: &'a SchemaKey, enum_: &SchemaEnum) -> KeyGenerator<'a> {
    let choice_type_name = key.name.to_pascal_case();
    let choice_enum = choice_enum(&choice_type_name, &enum_.values);
    KeyGenerator::new(key, Context::new_with_aux(&choice_type_name, choice_enum))
}

fn choice_enum(name: &str, enum_values: &[SchemaEnumValues]) -> proc_macro2::TokenStream {
    let variant_names = enum_values
        .iter()
        .map(|choice| choice.nick.as_str())
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
