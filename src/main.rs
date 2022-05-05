mod function;
mod types;

use heck::ToSnakeCase;

use std::{env, fs::File};

use self::{
    function::{Arg as FunctionArg, DelegateMethod, Function},
    types::{Key, Schema, SchemaList},
};

const USAGE_MESSAGE: &str = "usage: gsettings-codegen [FILE_PATH]";

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let mut args = env::args();
    args.next();
    let schema_file_path = args.next().ok_or_else(|| anyhow::anyhow!(USAGE_MESSAGE))?;
    let schema_file = File::open(&schema_file_path)?;
    let schema_list: SchemaList = serde_xml::from_reader(schema_file)?;

    anyhow::ensure!(schema_list.len() == 1, "only one schema is supported");

    let schema = schema_list
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("found no schema in the file"))?;

    println!("{}", generate_schema_code(schema));

    eprintln!("Successfully generated code at `{}`", schema_file_path);

    Ok(())
}

fn generate_key_code(key: &Key) -> anyhow::Result<String> {
    struct Context {
        arg_rust_type: String,
        ret_rust_type: String,
        call_name: String,
    }

    let context = match key.type_.as_str() {
        "i" => Context {
            arg_rust_type: "i32".into(),
            ret_rust_type: "i32".into(),
            call_name: "int".into(),
        },
        "s" => Context {
            arg_rust_type: "&str".into(),
            ret_rust_type: "glib::GString".into(),
            call_name: "string".into(),
        },
        "b" => Context {
            arg_rust_type: "bool".into(),
            ret_rust_type: "bool".into(),
            call_name: "boolean".into(),
        },
        type_ => anyhow::bail!("Unsupported type `{type_}`"),
    };

    let snake_case_key_name = key.name.to_snake_case();

    let mut buf = String::new();
    buf.push_str(
        &Function::new_method(&snake_case_key_name)
            .public(true)
            .ret_type(&context.ret_rust_type)
            .content(&format!(r#"self.0.{}("{}")"#, context.call_name, key.name))
            .generate(),
    );
    buf.push_str(
        &Function::new_method_with_args(
            &format!("set_{}", &snake_case_key_name),
            vec![FunctionArg::Other {
                name: "value".into(),
                type_: context.arg_rust_type.clone(),
            }],
        )
        .public(true)
        .ret_type("Result<(), glib::BoolError>")
        .content(&format!(
            r#"self.0.set_{}("{}", {})"#,
            context.call_name, key.name, "value"
        ))
        .generate(),
    );
    Ok(buf)
}

fn generate_schema_code(schema: &Schema) -> String {
    let mut buff = vec![
        format!(
            "// Generated with gsettings-codegen v{}",
            env!("CARGO_PKG_VERSION")
        ),
        String::new(),
        "#[derive(Debug, Clone)]".into(),
        "pub struct Settings(gio::Settings);".into(),
        String::new(),
        "impl Settings {".into(),
        Function::new("new")
            .public(true)
            .ret_type("Self")
            .content(&format!(r#"Self(gio::Settings::new("{}"))"#, schema.id))
            .generate(),
        String::new(),
        DelegateMethod::new_with_args(
            "create_action",
            vec![FunctionArg::Other {
                name: "key".into(),
                type_: "&str".into(),
            }],
        )
        .ret_type("gio::Action")
        .generate(),
        String::new(),
    ];

    for key in schema.keys.iter() {
        match generate_key_code(key) {
            Ok(code) => {
                buff.push(code);
                buff.push(String::new())
            }
            Err(err) => {
                log::info!("Skipped generating functions for `{}`: {}", key.name, err);
                continue;
            }
        }
    }

    buff.push("}".into());
    buff.push(String::new());

    buff.push("impl Default for Settings {".into());
    buff.push("fn default() -> Self {".into());
    buff.push("Self::new()".into());
    buff.push("}".into());
    buff.push("}".into());

    buff.join("\n")
}
