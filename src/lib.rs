mod as_dyn;
mod as_ref;
mod with_methods;

use proc_macro::TokenStream;

/// Generates an impl block for an enum containing the given methods,
/// where the method is a simple match over all the variants,
/// calling the same method on the matched variant's first field.
#[proc_macro_attribute]
pub fn with_methods(arg: TokenStream, input: TokenStream) -> TokenStream {
    with_methods::with_methods_impl(arg, input)
}

#[proc_macro_attribute]
pub fn as_dyn(arg: TokenStream, input: TokenStream) -> TokenStream {
    as_dyn::as_dyn_impl(arg, input)
}

#[proc_macro_attribute]
pub fn as_ref(arg: TokenStream, input: TokenStream) -> TokenStream {
    as_ref::as_ref_impl(arg, input)
}
