use crate::impl_basic_key;

impl_basic_key!(StringKey, "&str", "glib::GString", "string", "s");
impl_basic_key!(BoolKey, "bool", "bool", "boolean", "b");
impl_basic_key!(IntKey, "i32", "i32", "int", "i");
impl_basic_key!(UIntKey, "u32", "u32", "uint", "u");
