pub mod ast;
pub mod token;
pub mod scanner;
pub mod parser;
//use super::util;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    EOF,
    None,
    Message(String),
}