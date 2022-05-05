use super::Function;

/// Generates settings struct with new method
pub struct Settings {
    id: String,
    impl_codes: Vec<Function>,
}

impl Settings {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            impl_codes: Vec::new(),
        }
    }

    /// Adds code inside impl
    pub fn push_impl(&mut self, function: Function) {
        self.impl_codes.push(function);
    }

    /// Generate rust code
    pub fn generate(&self) -> String {
        let mut buf = String::new();

        buf.push_str("#[derive(Debug, Clone)] pub struct Settings(gio::Settings);");

        buf.push_str("impl Settings {");
        buf.push_str(
            &Function::new("new")
                .public(true)
                .ret_type("Self")
                .content(&format!(r#"Self(gio::Settings::new("{}"))"#, self.id))
                .generate(),
        );
        for code in &self.impl_codes {
            buf.push_str(&code.generate());
        }
        buf.push('}');

        buf.push_str("impl Default for Settings {");
        buf.push_str("fn default() -> Self { Self::new() }");
        buf.push('}');

        buf
    }
}
