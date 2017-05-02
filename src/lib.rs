#[macro_use]
mod macros;
pub mod util;

pub mod ast;
pub mod token;
pub mod scanner;
pub mod parser;
use std::result;
use std::path::Path;
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    None,
    EOF,
    Message(String),
    RefMessage(String, usize,usize,String),
}