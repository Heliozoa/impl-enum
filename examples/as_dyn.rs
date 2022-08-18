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
