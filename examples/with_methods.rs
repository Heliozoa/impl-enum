//! The variant of the writer is dynamically selected with an environment variable.
//! Using the macro, we get the convenience of a trait object with the performance of an enum.

use std::{
    env,
    fs::File,
    io::{Cursor, Write},
};

#[impl_enum::with_methods {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {}
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {}
}]
enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File { file: std::fs::File },
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
