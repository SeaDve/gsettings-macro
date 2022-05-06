# GSettings Macro

Create custom Settings struct with methods to access key.

The main purpose of this is to reduce the risk of mistyping a key and
reduce boilerplate rust code.

## Known issue

* Failing to compile on unknown variants
* Not updating when the gschema file is modified

## Todo list

* Add enum and flags support
* Add other common types support (`a{ss}`, etc.)
* Maybe include default, description, and summary in the generated code?
