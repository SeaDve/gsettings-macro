# GSettings Macro

Macro for easy GSettings key access

The main purpose of this is to reduce the risk of mistyping a key and
reduce boilerplate rust code.

## Known issue

* Failing to compile on unknown variants
  * Maybe skip generating them
* Not updating when the gschema file is modified
  * Use hacks like `include_str!`

## Todo list

* Add enum and flags support
* Add other common types support (`a{ss}`, etc.)
* Maybe include default, description, and summary in the generated code?
