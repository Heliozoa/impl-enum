Contains a proc macro attribute `impl_enum::with_methods` for generating methods on an enum that call the same method on each variant.

```rust
#[impl_enum::with_methods {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error>;
}]
enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File(File),
}
```

is equivalent to

```rust
enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File(File),
}

impl Writer {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        match self {
            Self::Cursor(cursor) => cursor.write_all(buf),
            Self::File(file) => file.write_all(buf),
        }
    }
}
```

With many variants and methods, maintaining such an impl can become tedious. The macro makes things easier to work with.

Variants with named fields and multiple fields are also supported, the method is always called on the first field and the rest are ignored. Enums with variants with no fields are currently not supported.

### Use cases

When the concrete type of some value depends on a runtime condition, a trait object or an enum with variants for each concrete type are the most natural choices, each having their own pros and cons. The cons of using an enum have to do mainly with usability, and so this crate aims to make them more convenient to use.
