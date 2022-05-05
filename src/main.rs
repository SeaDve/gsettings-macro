mod function;
mod key;
mod schema;
mod settings;

use std::{env, fs::File};

use self::{
    function::{Arg as FunctionArg, DelegateMethod, Function},
    schema::{Schema, SchemaList},
    settings::Settings,
};

const USAGE_MESSAGE: &str = "usage: gsettings-codegen [FILE_PATH]";

fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    args.next();
    let schema_file_path = args.next().ok_or_else(|| anyhow::anyhow!(USAGE_MESSAGE))?;
    let schema_file = File::open(&schema_file_path)?;
    let schema_list: SchemaList = serde_xml::from_reader(schema_file)?;

    anyhow::ensure!(
        schema_list.len() == 1,
        "only one schema is supported; found {}",
        schema_list.len()
    );

    let schema = schema_list
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("found no schema in the file"))?;

    let mut buf = String::new();
    buf.push_str(&format!(
        "// Generated with gsettings-codegen v{}",
        env!("CARGO_PKG_VERSION")
    ));
    buf.push('\n');
    buf.push('\n');
    buf.push_str(&generate_schema_code(schema));

    println!("{}", buf);

    eprintln!("Successfully generated code at `{}`", schema_file_path);

    Ok(())
}

fn generate_schema_code(schema: &Schema) -> String {
    let mut settings = Settings::new(&schema.id);

    settings.push_impl(
        DelegateMethod::new_with_args(
            "create_action",
            vec![FunctionArg::Other {
                name: "key".into(),
                type_: "&str".into(),
            }],
        )
        .ret_type("gio::Action")
        .into_inner(),
    );

    for key in schema.keys.iter() {
        for function in key.to_functions() {
            settings.push_impl(function);
        }
    }

    settings.generate()
}
