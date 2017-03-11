pub mod ascii;
use std::fmt::Debug;
use std::path::Path;
use std::str::from_utf8_unchecked;
use std::rc::Rc;
///// 用于记录节点位于原代码中的位置，展开(lineNo, column, srcSlice)。
//#[derive(Debug, Clone)]
//pub struct Pos<'a> {
//    pub line: usize,
//    pub column: usize,
//    pub str: &'a [u8],
//}
//
//impl<'a> Pos<'a> {
//    pub fn new(line: usize, column: usize, str: &'a [u8]) -> Pos<'a> {
//        Pos { line: line, column: column, str: str }
//    }
//}

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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Data,
    Symbol,
    DomTagStart,
    DomTagEnd,
    DomAttrStart,
    DomAttrEnd,
    DomCTag,
}

/// 定义的源码中最小词法的含义。
#[derive(Debug)]
pub struct Token<'a> {
    src: &'a Source,
    pub kind: TokenKind,
    pub offset: usize,
    pub len: usize,
}

impl<'a> Token<'a> {
    pub fn src_str(&self) -> &str {
        unimplemented!()
//        let s = self.src.content(self);
//        unsafe { from_utf8_unchecked(s) }
    }

    pub fn src_vec(&self) -> Vec<u8> {
        //let s = self.src.content(self);
        let mut arr: Vec<u8> = Vec::new();
        //arr.extend_from_slice(s);
        return arr;
    }

    pub fn new(src: &Source, kind: TokenKind, offset: usize, len: usize) -> Token {
        Token { src: src, kind: kind, offset: offset, len: len }
    }
}
//#[derive(Debug)]
//pub struct Source<'a>{
//    pub content: &'a [u8],
//}

pub trait Source: Debug {//+Sized
    fn line(&self, tok: &Token) -> usize;
    fn column(&self, tok: &Token) -> usize;
    fn filename(&self) -> &Path;
    fn content(&self, tok: &Token) -> &[u8];
}