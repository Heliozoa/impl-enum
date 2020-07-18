use std::env;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;

#[impl_enum::with_methods {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error>;
}]
enum Writer {
    Cursor(Cursor<Vec<u8>>),
    File(File),
}

fn get_writer() -> Writer {
    if env::var("USE_CURSOR").is_ok() {
        Writer::Cursor(Cursor::new(vec![]))
    } else {
        Writer::File(File::create("some.log").unwrap())
    }
}

fn main() {
    let mut writer = get_writer();
    writer.write_all("hello!".as_bytes()).unwrap();
}
