pub mod ascii;

use std::path::Path;
use std::str::from_utf8_unchecked;
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
#[derive(Debug, Clone)]
pub struct Token<'a> {
    src: &'a Source,
    pub kind: TokenKind,
    pub offset: usize,
    pub len: usize,
//    pub column: usize,
//    pub filename: &'a Path,
//    pub str: &'a [u8],
//    pub kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn src_str(&self) -> &str {
        unsafe { from_utf8_unchecked(self.str) }
    }

    pub fn src_vec(&self) -> Vec<u8> {
        let mut arr: Vec<u8> = Vec::new();
        arr.extend_from_slice(self.str);
        return arr;
    }

    pub fn new(line: usize, column: usize, filename: &'a Path, str: &'a [u8], kind: TokenKind) -> Token<'a> {
        Token { line: line, column: column, filename: filename, str: str, kind: kind }
    }
}

pub trait Source<'a> {
    fn line(&self, tok: &'a Token) -> usize;
    fn column(&self, tok: &'a Token) -> usize;
    fn filename(&self) -> &'a Path;
    fn content(&self, tok: &'a Token) -> &'a [u8];
}