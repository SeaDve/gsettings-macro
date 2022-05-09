use heck::ToPascalCase;

use super::{Context, KeyGenerator, SchemaEnum, SchemaKey};

pub fn key_generator<'a>(key: &'a SchemaKey, enum_: &SchemaEnum) -> KeyGenerator<'a> {
    let enum_name = key.name.to_pascal_case();
    let enum_token_stream = super::new_variant_enum(
        &enum_name,
        &enum_
            .values
            .iter()
            .map(|choice| choice.nick.as_str())
            .collect::<Vec<_>>(),
    );
    KeyGenerator::new(key, Context::new_with_aux(&enum_name, enum_token_stream))
}
