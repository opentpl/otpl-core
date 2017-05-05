use token::{Token, TokenKind};
use {Error, Result, NoneResult};
use util::VecSliceCompare;
use super::Parser;

/// 定义用于解析过程中的断点。
pub struct BreakPoint {
    /// 是否保留已检查的token
    pub keep: bool,
    /// 用于测试的类别
    pub kind: TokenKind,
    /// 用于测试的值得集合
    pub values: Vec<Vec<u8>>,
}

impl BreakPoint {
    pub fn new(keep: bool, kind: TokenKind, values: Vec<Vec<u8>>) -> BreakPoint {
        BreakPoint { keep: keep, kind: kind, values: values }
    }

    pub fn build(breaks: Vec<BreakPoint>) -> Box<(FnMut(&mut Parser) -> NoneResult)> {
        return Box::new(move |parser: &mut Parser| -> NoneResult {
            let mut found;
            for point in &breaks {
                if point.values.is_empty() {
                    continue;
                }
                found = true;
                let mut buf: Vec<Token> = vec![];
                for value in &point.values {
                    match parser.take().and_then(|tok| -> NoneResult{
                        if (point.kind != TokenKind::Any && &point.kind != tok.kind())
                            || !value.compare(parser.tokenizer.source().content(&tok)) {
                            buf.push(tok);
                            return Err(Error::None);
                        }
                        buf.push(tok);
                        return Error::ok();
                    }) {
                        Ok(_) => {}
                        Err(Error::None) => { found = false; }
                        err => { return err; }
                    }

                    if !found { break; }
                }

                if !found || point.keep {
                    while !buf.is_empty() {
                        parser.back(buf.pop().unwrap());
                    }
                }

                if found { return Error::ok(); }
            }
            return Err(Error::None);
        });
    }
}