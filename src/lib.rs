mod generators;
mod imp;
mod schema;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

/// Needs `gio` in scope.
///
/// Not specifying the id in the attribute will require the id in the [`new`] constructor.
/// Additionally, it will not implement [`Default`].
#[proc_macro_attribute]
#[proc_macro_error]
pub fn gen_settings(attr: TokenStream, item: TokenStream) -> TokenStream {
    imp::impl_gen_settings(attr, item)
}
