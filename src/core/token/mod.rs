pub mod ascii;

/// 标记的种类
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    EOF,
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
}

