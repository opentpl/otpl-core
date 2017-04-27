use std::fmt::Debug;
use core::token::Token;
use super::Source;

pub trait Scanner: Debug {
    fn back(&mut self, tok: Token);
    fn scan(&mut self) -> Result<Token, String>;
    fn source(&self) -> &Source;
}