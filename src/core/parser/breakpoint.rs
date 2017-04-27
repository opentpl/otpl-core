use super::Parser;
use core::token::{Token, TokenKind}; //, Source
use util::VecSliceCompare;
//use util::Queue;

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

    pub fn build(breaks: Vec<BreakPoint>) -> Box<(FnMut(&mut Parser) -> bool)> {
        return Box::new(move |owner: &mut Parser| -> bool {
            let mut found;
            for point in &breaks {
                if point.values.is_empty() {
                    continue;
                }
                found = true;
                let mut buf: Vec<Token> = vec![];
                for value in &point.values {
                    if let Option::Some(tok) = owner.take() {
                        if (point.kind != TokenKind::Any && &point.kind != tok.kind()) || !value.compare(owner.scanner.source().content(&tok)) {
                            //
                            found = false;
                        }
                        buf.push(tok);
                    } else {
                        found = false;
                    }
                    if !found { break; }
                }
                if !found || point.keep {
                    while !buf.is_empty() {
                        owner.back(buf.pop().unwrap());
                    }
                }

                if found {
                    return found;
                }
            }
            return false;
        });
    }
}