//! Macros that make it more convenient to work with enums with variants that
//! all implement the same trait(s).
//!
//! [macro@with_methods] allows you to easily delegate method calls to enum variants:
//! ```
#![doc = include_str!("../examples/with_methods.rs")]
//! ```
//!
//! [macro@as_dyn] allows you to treat the enum as a trait object when necessary:
//! ```
#![doc = include_str!("../examples/as_dyn.rs")]
//! ```

#[cfg(feature = "as_dyn")]
mod as_dyn;
#[cfg(feature = "with_methods")]
mod with_methods;

use proc_macro::TokenStream;
use syn::{spanned::Spanned, Error, Field, Fields, Variant};

/// Generates methods for an enum that match on the enum
/// and call given the method with the variant's first field.
///
/// Takes a list of whitespace separated function signatures as its arguments.
///
/// # Example
/// ```
#[doc = include_str!("../examples/with_methods.rs")]
/// ```
/// The macro generates an impl block equivalent to
/// ```
/// # use std::io::Write;
/// # enum Writer { Cursor(std::fs::File), File { file: std::fs::File } }
/// impl Writer {
///     fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
///         match self {
///             Self::Cursor(first, ..) => first.write_all(buf),
///             Self::File { file, .. } => file.write_all(buf),
///         }
///     }
///     pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
///         match self {
///             Self::Cursor(first, ..) => first.write(buf),
///             Self::File { file, .. } => file.write(buf),
///         }
///     }
/// }
/// ```
#[cfg(feature = "with_methods")]
#[proc_macro_attribute]
pub fn with_methods(args: TokenStream, input: TokenStream) -> TokenStream {
    with_methods::with_methods_impl(args, input)
}

/// Generates methods for an enum that match on the enum
/// and return the variant's first field as a trait object.
///
/// Takes a comma-separated list of traits as an argument.
/// The name of the trait is snake_cased for the method names.
/// For example, for the trait `ExampleTrait`  it would generate
/// ```
/// # trait ExampleTrait {}
/// # struct S;
/// # impl S {
/// fn as_dyn_example_trait(&self) -> &dyn ExampleTrait
/// # { unimplemented!() }
/// fn as_dyn_example_trait_mut(&mut self) -> &mut dyn ExampleTrait
/// # { unimplemented!() }
/// fn into_dyn_example_trait(self) -> Box<dyn ExampleTrait>
/// # { unimplemented!() }
/// # }
/// ```
///
/// # Example
/// ```
#[doc = include_str!("../examples/as_dyn.rs")]
/// ```
/// The macro generates an impl block equivalent to
/// ```
/// # use std::io::Write;
/// # enum Writer { Cursor(std::fs::File), File { file: std::fs::File } }
/// impl Writer {
///     fn as_dyn_write(&self) -> &dyn Write {
///         match self {
///             Self::Cursor(first, ..) => first as &dyn Write,
///             Self::File { file, .. } => file as &dyn Write,
///         }
///     }
///     fn as_dyn_write_mut(&mut self) -> &mut dyn Write {
///         match self {
///             Self::Cursor(first, ..) => first as &mut dyn Write,
///             Self::File { file, .. } => file as &mut dyn Write,
///         }
///     }
///     fn into_dyn_write(self) -> Box<dyn Write> {
///         match self {
///             Self::Cursor(first, ..) => Box::new(first) as Box<dyn Write>,
///             Self::File { file, .. } => Box::new(file) as Box<dyn Write>,
///         }
///     }
/// }
/// ```
#[cfg(feature = "as_dyn")]
#[proc_macro_attribute]
pub fn as_dyn(args: TokenStream, input: TokenStream) -> TokenStream {
    as_dyn::as_dyn_impl(args, input)
}

fn first_field(variant: &Variant) -> syn::Result<&Field> {
    match &variant.fields {
        Fields::Named(fields) => fields.named.first(),
        Fields::Unnamed(fields) => fields.unnamed.first(),
        Fields::Unit => {
            return Err(Error::new(
                variant.span(),
                "Unit variants are not supported",
            ))
        }
    }
    .ok_or_else(|| {
        Error::new(
            variant.fields.span(),
            "Enum variants must have at least one field",
        )
    })
}
