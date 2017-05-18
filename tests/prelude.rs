pub extern crate otpl;
//pub use self::otpl;
pub use self::otpl::parser;
pub use self::otpl::parser::Parser;
pub use self::otpl::scanner::{BytesScanner, Source};
pub use self::otpl::ast;
pub use self::otpl::ast::{Visitor,Node,NodeList};
pub use self::otpl::token;
pub use self::otpl::token::Token;
use std::fs::OpenOptions;
use std::path::{Path};
use std::io;
use std::io::prelude::*;
pub fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
    return OpenOptions::new().read(true).open(path)
        .and_then(|mut f| -> io::Result<Vec<u8>>{
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).unwrap();
            return Ok(buf);
        }).expect("打开文件失败");
}