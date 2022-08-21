# impl-enum

[![Crates.io](https://img.shields.io/crates/v/impl-enum)](https://crates.io/crates/impl-enum)
[![docs.rs](https://img.shields.io/docsrs/impl-enum)](https://docs.rs/impl-enum)
[![Crates.io](https://img.shields.io/crates/l/impl-enum)](https://choosealicense.com/licenses/mpl-2.0/)

Contains proc macro attributes `with_methods` and `as_dyn` that make using enums like trait objects more convenient.

## Use cases

When the concrete type of some value depends on a runtime condition, a trait object or an enum with variants for each concrete type are the most natural choices, each having their own pros and cons. One of the cons of using an enum is that it can be inconvenient to use. The trait object method can also be problematic if you want to use types that cannot be turned into trait objects, or when working with types and traits defined in other crates. This crate aims to make such enums more convenient to use.

## [`with_methods`](https://docs.rs/impl-enum/latest/impl_enum/attr.with_methods.html)

```rust
//! The variant of the writer is dynamically selected with an environment variable.
//! Using the macro, we can use the enum with the convenience of a trait object.

use std::{
    env,
    fs::File,
    io::{Cursor, Write},
};

#[impl_enum::with_methods {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>
    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
}]
pub enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File { file: File },
}

fn get_writer() -> Writer {
    if let Ok(path) = env::var("WRITER_FILE") {
        Writer::File {
            file: File::create(path).unwrap(),
        }
    } else {
        Writer::Cursor(Cursor::new(vec![]))
    }
}

fn main() {
    let mut writer = get_writer();
    writer.write_all(b"hello!").unwrap();
}
```

The macro generates an impl block equivalent to

```rust
impl Writer {
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Cursor(first, ..) => first.write_all(buf),
            Self::File { file, .. } => file.write_all(buf),
        }
    }
    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Cursor(first, ..) => first.write(buf),
            Self::File { file, .. } => file.write(buf),
        }
    }
}
```

## [`as_dyn`](https://docs.rs/impl-enum/latest/impl_enum/attr.as_dyn.html)
```rust
//! The variant of the writer is dynamically selected with an environment variable.
//! Using the macro, we can conveniently turn the enum into a trait object when necessary.

use std::{
    fmt::Debug,
    fs::File,
    io::{Cursor, Write},
};

#[impl_enum::as_dyn(Debug, Write)]
pub enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File { file: File },
}

fn get_writer() -> Writer {
    if let Ok(path) = std::env::var("WRITER_FILE") {
        Writer::File {
            file: File::create(path).unwrap(),
        }
    } else {
        Writer::Cursor(Cursor::new(vec![]))
    }
}

fn main() {
    let mut writer = get_writer();

    let dyn_debug = writer.as_dyn_debug();
    println!("{:?}", dyn_debug);

    let dyn_writer_mut = writer.as_dyn_write_mut();
    dyn_writer_mut.write_all(b"hello!").unwrap();

    let box_dyn_debug = writer.into_dyn_debug();
    println!("{:?}", box_dyn_debug);
}
```

The macro generates an impl block equivalent to

```rust
impl Writer {
    fn as_dyn_write(&self) -> &dyn Write {
        match self {
            Self::Cursor(first, ..) => first as &dyn Write,
            Self::File { file, .. } => file as &dyn Write,
        }
    }
    fn as_dyn_write_mut(&mut self) -> &mut dyn Write {
        match self {
            Self::Cursor(first, ..) => first as &mut dyn Write,
            Self::File { file, .. } => file as &mut dyn Write,
        }
    }
    fn into_dyn_write(self) -> Box<dyn Write> {
        match self {
            Self::Cursor(first, ..) => Box::new(first) as Box<dyn Write>,
            Self::File { file, .. } => Box::new(file) as Box<dyn Write>,
        }
    }
}
```

## Alternatives
- https://crates.io/crates/ambassador

## License
Licensed under the Mozilla Public License Version 2.0.
