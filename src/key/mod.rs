mod basic;

use heck::ToSnakeCase;

use super::{Function, FunctionArg};

#[typetag::serde(tag = "type")]
pub trait Key: std::fmt::Debug {
    /// Name of they key as defined in the schema
    fn name(&self) -> &str;

    /// Setter function content
    ///
    /// Note: Use the string `value` as the arg
    fn setter_content(&self) -> String;

    /// Getter function content
    fn getter_content(&self) -> String;

    /// Setter function arg type or the type of `value`
    fn arg_type(&self) -> &str;

    /// Getter function return type
    fn ret_type(&self) -> &str;

    /// Create function from this
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
