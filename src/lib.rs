#[macro_use]
mod macros;
pub mod util;

pub mod ast;
pub mod token;
pub mod scanner;
pub mod parser;

use std::result;

#[derive(Debug)]
pub enum Error {
    None,
    EOF,
    Ok,
    Message(String),
    RefMessage(String, usize, usize, String),
    Scan(String, usize),
    Parse(String, usize),
    Visit(String, usize),
}

pub type Result<T> = result::Result<T, Error>;
pub type NoneResult = Result<()>;

impl Error {
    pub fn ok() -> NoneResult { Ok(()) }
    pub fn eof_none() -> NoneResult { Err(Error::EOF) }


    pub fn unwrap(&self, source: &scanner::Source) {
        match self {
            &Error::None | &Error::Ok => {}
            &Error::Parse(ref msg, ref offset) => {
                panic!("Parsing failed at: {}({}:{}): {}", source.filename().to_str().unwrap(), source.line(*offset), source.column(*offset), msg)
            }
            &Error::Scan(ref msg, ref offset) => {
                panic!("Scanning failed at: {}({}:{}): {}", source.filename().to_str().unwrap(), source.line(*offset), source.column(*offset), msg)
            }
            &Error::Visit(ref msg, ref offset) => {
                panic!("Visiting failed at: {}({}:{}): {}", source.filename().to_str().unwrap(), source.line(*offset), source.column(*offset), msg)
            }
            _ => {
                panic!("{:?}", self)
            }
        }
    }
}


