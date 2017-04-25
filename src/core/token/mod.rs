pub mod ascii;

use std::fmt::Debug;
use std::path::Path;
use std::str::from_utf8_unchecked;

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

    pub fn content_str<'a, T: Source>(&'a self, src: &'a T) -> &'a str {
        let s = src.content(self);
        return unsafe { from_utf8_unchecked(s) };
    }

    pub fn content_vec<T: Source>(&self, src: &T) -> Vec<u8> {
        let s = src.content(self);
        let mut arr: Vec<u8> = Vec::new();
        arr.extend_from_slice(s);
        return arr;
    }
}

/// 定义的要解析的输入源。
pub trait Source: Debug {
    /// 获取给定 `Token` 的用于定位源的行号.
    fn line(&self, offset: usize) -> usize;
    /// 获取给定 `Token` 的用于定位源的行的开始位置.
    fn column(&self, offset: usize) -> usize;
    /// 获取给定 `Token` 的输入源文件名.
    /// 注意：该文件名只是用于错误定位的提示。
    fn filename(&self) -> &Path;
    /// 获取给定 `Token` 的内容.
    fn content(&self, tok: &Token) -> &[u8];
    /// 获取源
    fn source(&self) -> &[u8];
}