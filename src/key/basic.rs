use crate::impl_default_key;

impl_default_key!(StringKey, "&str", "glib::GString", "string", "s");
impl_default_key!(BoolKey, "bool", "bool", "boolean", "b");
impl_default_key!(IntKey, "i32", "i32", "int", "i");
