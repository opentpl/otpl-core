pub mod ascii;

use std::fmt::Debug;
use std::path::Path;
use std::str::from_utf8_unchecked;
//use std::rc::Rc;


//#[derive(Debug, Clone)]
//pub enum Token<'a> {
//    None,
//    LSS(usize, usize),
//    LEQ(usize, usize),
//    GTR(usize, usize),
//    GEQ(usize, usize),
//    Symbol(usize, usize, u8),
//    Data(usize, usize, Vec<u8>),
//    StmtStart(usize, usize, Vec<u8>),
//    StmtEnd(usize, usize, Vec<u8>),
//    LiteralBoundary(usize, usize, bool),
//    Literal(usize, usize, Vec<u8>),
//    Comments(usize, usize, Vec<u8>),
//    DomTagStart(Pos<'a>),
//    DomTagEnd(Pos<'a>),
//    DomTagAttrStart(Pos<'a>),
//    DomTagAttrEnd(Pos<'a>),
//}

/// 标记的种类
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Any,
    Data,
    Symbol,
    String,
    Int,
    Ident,
    DomTagStart,
    DomTagEnd,
    DomAttrStart,
    DomAttrEnd,
    DomCTag,
    DomComment,
    LDelimiter,
    RDelimiter,
    Literal,
}

/// 定义的源码中最小词法的含义。
/// Token( [`Source`] , `TokenKind`, start offset, end offset)
#[derive(Debug)]
pub struct Token(pub TokenKind, pub usize, pub usize);

impl Token {
    pub fn kind(&self) -> &TokenKind {
        &self.0
    }

    pub fn content_str<'a,T: Source>(&'a self, src: &'a T) -> &'a str {
        let s = src.content(self);
        return unsafe { from_utf8_unchecked(s) };
    }

    pub fn content_vec<T: Source>(&self, src: &T) -> Vec<u8> {
        let s = src.content(self);
        let mut arr: Vec<u8> = Vec::new();
        arr.extend_from_slice(s);
        return arr;
    }

//    pub fn new(src: &Source) -> Token {
//        Token { src: src }
//    }
}

/// 定义的要解析的输入源。
pub trait Source: Debug {

    fn as_ref(&self) -> &Self{
        self
    }

    //+Sized
    /// 获取给定 `Token` 的用于定位源的行号.
    fn line(&self, offset: usize) -> usize;
    /// 获取给定 `Token` 的用于定位源的行的开始位置.
    fn column(&self, offset: usize) -> usize;
    /// 获取给定 `Token` 的输入源文件名.
    /// 注意：该文件名只是用于错误提示的定位。
    fn filename(&self) -> &Path;
    /// 获取给定 `Token` 的内容.
    /// ```
    /// return src[tok.offset..tok.offset.3]
    /// ```
    fn content(&self, tok: &Token) -> &[u8];
    fn source(&self) -> &[u8];
    fn get(offset: usize) -> u8;
}