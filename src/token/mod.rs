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
/// Token([`TokenKind`], start-offset, end-offset, pos)
#[derive(Debug, Clone)]
pub struct Token(pub TokenKind, pub usize, pub usize, pub usize);

impl Token {
    pub fn kind(&self) -> &TokenKind {
        &self.0
    }
}

impl PartialEq<Token> for Token {
    fn eq(&self, other: &Token) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}