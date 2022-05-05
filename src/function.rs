#[derive(Debug, Clone)]
pub enum Arg {
    SelfRef,
    Other { name: String, type_: String },
}

/// Generates function rust code
pub struct Function {
    name: String,
    args: Vec<Arg>,
    is_pub: Option<bool>,
    ret_type: Option<String>,
    content: Option<String>,
}

impl Function {
    /// Function with no args
    pub fn new(name: &str) -> Self {
        Self::new_with_args(name, Vec::new())
    }

    /// Function with args
    pub fn new_with_args(name: &str, args: Vec<Arg>) -> Self {
        Self {
            name: name.to_string(),
            args,
            is_pub: None,
            ret_type: None,
            content: None,
        }
    }

    /// Function with `&self` parameter
    pub fn new_method(name: &str) -> Self {
        Self::new_method_with_args(name, Vec::new())
    }

    /// Function with `&self` parameter with additional args
    pub fn new_method_with_args(name: &str, mut args: Vec<Arg>) -> Self {
        Self::new_with_args(name, {
            let mut arg_list = vec![Arg::SelfRef];
            arg_list.append(&mut args);
            arg_list
        })
    }

    /// Whether the function has pub prefix
    pub fn public(mut self, value: bool) -> Self {
        self.is_pub = Some(value);
        self
    }

    /// Rust return type
    pub fn ret_type(mut self, value: impl Into<String>) -> Self {
        self.ret_type = Some(value.into());
        self
    }

    /// Actual content of the function
    pub fn content(mut self, value: impl Into<String>) -> Self {
        self.content = Some(value.into());
        self
    }

    /// Generate rust code
    pub fn generate(&self) -> String {
        let args = self
            .args
            .iter()
            .map(|arg| match arg {
                Arg::SelfRef => "&self".to_string(),
                Arg::Other { name, type_ } => {
                    format!("{name}: {type_}")
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        let mut buf = String::new();

        if self.is_pub.unwrap_or(false) {
            buf.push_str("pub");
            buf.push(' ');
        }

        buf.push_str("fn");

        buf.push(' ');
        buf.push_str(&self.name);
        buf.push('(');
        buf.push_str(&args);
        buf.push(')');
        buf.push(' ');

        if let Some(ref ret_type) = self.ret_type {
            buf.push_str("->");
            buf.push(' ');
            buf.push_str(ret_type);
            buf.push(' ');
        }

        buf.push('{');
        if let Some(ref content) = self.content {
            buf.push_str(content);
        }
        buf.push('}');

        buf
    }
}

/// Assumes that inner can be accessed with `self.0`
pub struct DelegateMethod(Function);

impl DelegateMethod {
    pub fn new_with_args(name: &str, args: Vec<Arg>) -> Self {
        let content_args = args
            .iter()
            .filter_map(|arg| match arg {
                Arg::SelfRef => None,
                Arg::Other { name, .. } => Some(name.to_string()),
            })
            .collect::<Vec<_>>()
            .join(",");

        Self(
            Function::new_method_with_args(name, args)
                .public(true)
                .content(format!("self.0.{}({})", name, content_args)),
        )
    }

    /// Rust return type
    pub fn ret_type(self, value: impl Into<String>) -> Self {
        Self(self.0.ret_type(value))
    }

    /// Convert into function
    pub fn into_inner(self) -> Function {
        self.0
    }
}
