# impl-enum

[![Crates.io](https://img.shields.io/crates/v/impl-enum)](https://crates.io/crates/impl-enum)
[![docs.rs](https://img.shields.io/docsrs/impl-enum)](https://docs.rs/impl-enum)
[![Crates.io](https://img.shields.io/crates/l/impl-enum)](https://choosealicense.com/licenses/mpl-2.0/)

Contains a proc macro attribute `impl_enum::with_methods` for generating methods on an enum that call the same method on each variant.

## Use cases

When the concrete type of some value depends on a runtime condition, a trait object or an enum with variants for each concrete type are the most natural choices, each having their own pros and cons. The cons of using an enum have to do mainly with usability, and so this crate aims to make them more convenient to use. The trait object method can also be problematic if you want to use types that cannot be turned into trait objects, or when working with types and traits defined in other crates.

## Example

```rust
// The variant of the writer is dynamically selected with an environment variable.
// Using the macro, we get the convenience of a trait object with the performance of an enum.

use std::env;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;

#[impl_enum::with_methods {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {}
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {}
}]
enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File(File),
}

fn get_writer() -> Writer {
    if let Ok(path) = env::var("WRITER_FILE") {
        Writer::File(File::create(path).unwrap())
    } else {
        Writer::Cursor(Cursor::new(vec![]))
    }
}

fn main() {
    let mut writer = get_writer();
    writer.write_all(b"hello!").unwrap();
}
```

The macro generates an impl block for the Writer enum equivalent to

```rust
impl Writer {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        match self {
            Self::Cursor(cursor) => cursor.write_all(buf),
            Self::File(file) => file.write_all(buf),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            Self::Cursor(cursor) => cursor.write(buf),
            Self::File(file) => file.write(buf),
        }
    }
}
```

This would be simple enough to write manually in this case, but with many variants and methods, maintaining such an impl can become tedious. The macro is intended to make such an enum easier to work with.

Variants with named fields and multiple fields are also supported, the method is always called on the first field and the rest are ignored. Enums with variants with no fields are currently not supported.

## License
Licensed under the Mozilla Public License Version 2.0.
