pub mod ascii;

use std::str::from_utf8_unchecked;

/// 标记的种类
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    /// 模板结束标志
    EOF,
    /// 用于辅助处理，标示忽略
    Ignore,
    ///TODO: 考虑用 Literal 代替？
    Data,
    /// 符号
    Symbol,
    /// 字符串
    String,
    /// 整数
    Int,
    /// 标识符
    Identifier,
    /// DOM标签的开始
    DomTagStart,
    /// DOM标签的结束
    DomTagEnd,
    /// DOM标签属性的开始
    DomAttrStart,
    /// DOM标签属性的结束
    DomAttrEnd,
    /// DOM标签属性的结束
    DomAttrValue,
    /// DOM标签的闭合部分，如：</div>
    DomCTag,
    /// DOM注释
    DomComment,
    /// TPL代码开始边界符
    LDelimiter,
    /// TPL代码结束边界符
    RDelimiter,
    /// 字面量
    Literal,
}

/// 定义的源码中最小词法的含义。
/// Token([`TokenKind`], start-offset, end-offset)
#[derive(Debug, Clone)]
pub struct Token(pub TokenKind, pub usize, pub Vec<u8>);

impl Token {
    pub fn kind(&self) -> &TokenKind {
        &self.0
    }

    pub fn offset(&self) -> usize {
        self.1
    }

    pub fn value(&self) -> &[u8] {
        self.2.as_slice()
    }

    pub fn value_str(&self) -> &str {
        let s = self.value();
        return unsafe { from_utf8_unchecked(s) };
    }

    pub fn empty() -> Token {
        Token(TokenKind::Ignore, 0, vec![])
    }
}

impl PartialEq<Token> for Token {
    fn eq(&self, other: &Token) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}