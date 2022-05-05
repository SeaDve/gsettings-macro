use crate::impl_basic_key;

impl_basic_key!(BooleanKey, "bool", "bool", "boolean", "b");

impl_basic_key!(StringKey, "&str", "glib::GString", "string", "s");
impl_basic_key!(StringVecKey, "&[&str]", "Vec<glib::GString>", "strv", "as");

impl_basic_key!(IntKey, "i32", "i32", "int", "i");
impl_basic_key!(UIntKey, "u32", "u32", "uint", "u");

impl_basic_key!(Int64Key, "i64", "i64", "int64", "x");
impl_basic_key!(UInt64Key, "u64", "u64", "uint64", "t");

impl_basic_key!(DoubleKey, "f64", "f64", "double", "d");
