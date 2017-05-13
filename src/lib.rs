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
    RefMessage(String, usize,usize,String),
}

pub type Result<T> = result::Result<T, Error>;

pub type NoneResult = Result<()>;

impl Error{
    pub fn ok() ->NoneResult{ Ok(()) }
    pub fn eof_none() ->NoneResult{ Err(Error::EOF) }
}