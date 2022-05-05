mod basic;

use heck::ToSnakeCase;

use super::{Function, FunctionArg};

#[typetag::serde(tag = "type")]
pub trait Key: std::fmt::Debug {
    fn name(&self) -> &str;

    fn setter_content(&self) -> String;

    fn getter_content(&self) -> String;

    fn arg_type(&self) -> &str;

    fn ret_type(&self) -> &str;

    fn to_functions(&self) -> Vec<Function> {
        let getter = Function::new_method(&self.name().to_snake_case())
            .public(true)
            .ret_type(self.ret_type())
            .content(&self.getter_content());

        let setter = Function::new_method_with_args(
            &format!("set_{}", &self.name().to_snake_case()),
            vec![FunctionArg::Other {
                name: "value".into(),
                type_: self.arg_type().to_string(),
            }],
        )
        .public(true)
        .ret_type("Result<(), glib::BoolError>")
        .content(&self.setter_content());

        vec![getter, setter]
    }
}

/// Needs the following parameters:
/// - name: Name of Struct
/// - arg_type: Argument type used in setter (`&str`, `i64`, etc.)
/// - ret_type: Argument type used in getter (`glib::GString`, `i64`, etc.)
/// - call_name: What method to call in [`gio::Settings`] (`int`, `boolean`, etc.)
/// - variant_type: [`glib::Variant`] type string (`i`, `b`, etc.)
#[macro_export]
macro_rules! impl_basic_key {
    ($name:ident, $arg_type:expr, $ret_type:expr, $call_name:expr, $variant_type:expr) => {
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
