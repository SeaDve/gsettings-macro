/// Needs the following parameters:
/// - name: Name of Struct
/// - arg_type: Argument type used in setter (`&str`, `i64`, etc.)
/// - ret_type: Argument type used in getter (`glib::GString`, `i64`, etc.)
/// - call_name: What method to call in [`gio::Settings`] (`int`, `boolean`, etc.)
/// - variant_type: [`glib::Variant`] type string (`i`, `b`, etc.)
macro_rules! impl_basic_key {
    ($name:ident, $arg_type:literal, $ret_type:literal, $call_name:literal, $variant_type:literal) => {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct $name {
            name: String,
        }

        #[typetag::serde(name = $variant_type)]
        impl crate::key::Key for $name {
            fn name(&self) -> &str {
                self.name.as_str()
            }

            fn setter_content(&self) -> String {
                format!(r#"self.0.set_{}("{}", value)"#, $call_name, self.name())
            }

            fn getter_content(&self) -> String {
                format!(r#"self.0.{}("{}")"#, $call_name, self.name())
            }

            fn arg_type(&self) -> &str {
                $arg_type
            }

            fn ret_type(&self) -> &str {
                $ret_type
            }
        }
    };
}

impl_basic_key!(BooleanKey, "bool", "bool", "boolean", "b");

impl_basic_key!(StringKey, "&str", "glib::GString", "string", "s");
impl_basic_key!(StringVecKey, "&[&str]", "Vec<glib::GString>", "strv", "as");

impl_basic_key!(IntKey, "i32", "i32", "int", "i");
impl_basic_key!(UIntKey, "u32", "u32", "uint", "u");

impl_basic_key!(Int64Key, "i64", "i64", "int64", "x");
impl_basic_key!(UInt64Key, "u64", "u64", "uint64", "t");

impl_basic_key!(DoubleKey, "f64", "f64", "double", "d");
