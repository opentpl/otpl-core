use super::ast;
use super::ast::Node;
use super::ast::NodeList;
use super::token::TokenKind;
use super::token::Token;
use super::scanner::Scanner;
use super::token::ascii;
//use std::ops::Index;
use std::str::from_utf8_unchecked;
//use std::cell::RefCell;
//use std::sync::Arc;

trait VecSliceCompare<T: PartialEq> {
    fn compare(&self, s: &[T]) -> bool;
}

impl<T: PartialEq> VecSliceCompare<T> for Vec<T> {
    fn compare(&self, s: &[T]) -> bool {
        if self.len() != s.len() {
            return false;
        }
        for i in 0..self.len() {
            if self[i] != s[i] {
                return false;
            }
        }
        return true;
    }
}

pub struct BreakPoint {
    /// 是否保留已检查的token
    pub keep: bool,
    pub values: Vec<Vec<u8>>,
}

impl BreakPoint {
    pub fn new(keep: bool, values: Vec<Vec<u8>>) -> BreakPoint {
        BreakPoint { keep: keep, values: values }
    }
}


pub struct Parser<'a> {
    scanner: Scanner<'a>,
    token_buf: Vec<Token<'a>>,
    breaks: Vec<BreakPoint>,
    break_checkers: Vec<Box<(Fn(&mut Parser) -> bool)>>,
}


impl<'a> Parser<'a> {
    pub fn new() -> Parser<'a> {
        let mut scanner = Scanner::new("<div id=\"te\\\"st\">".as_bytes(), "{{".as_bytes(), "}}".as_bytes());
        return Parser {
            scanner: scanner,
            token_buf: vec![],
            breaks: vec![],
            break_checkers: vec![],
        };
    }

    //    fn peek(&mut self) -> Option<&mut Token>{
    //        if self.token_buf.is_empty(){
    //            let tok = self.scanner.scan();
    //            if let Token::None = tok {
    //                return Option::None;
    //            }
    //            self.token_buf.insert(0, self.scanner.scan());
    //        }
    //        let tok = &self.token_buf[self.token_buf.len() - 1];
    //        return Option::Some(&tok.clone());
    //    }

    fn take(&mut self) -> Option<Token<'a>> {
        if self.token_buf.is_empty() {
            return self.scanner.scan();
        }
        return self.token_buf.pop();
    }

    fn back(&mut self, tok: Token<'a>) {
        self.token_buf.insert(0, tok);
    }

    fn skip_symbol(&mut self, symbol: Vec<u8>) -> bool {
        if let Some(tok) = self.take() {
            match tok.kind {
                TokenKind::DomTagAttrStart => {
                    if symbol.compare(tok.str) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        return false;
    }

    fn set_breakpoint(&mut self, checker: Box<(Fn(&mut Parser) -> bool)>) {
        self.break_checkers.push(checker);
    }

    fn pop_breakpoint(&mut self) -> Option<Box<(Fn(&mut Parser) -> bool)>> {
        self.break_checkers.pop()
    }

    fn check_breakpoint(&mut self) -> bool {
        if self.break_checkers.is_empty() {
            return false;
        }
        let checker = self.break_checkers.pop().unwrap();
        let result = checker(self);
        self.break_checkers.push(checker);
        return result;
    }

    /// 期望一个类型。如果未找到则产生一个错误。
    fn expect_type(&mut self, kind: TokenKind) -> Option<Token<'a>> {
        if let Some(tok) = self.take() {
            if tok.kind == kind {
                return Some(tok);
            }
            debug!("expected type {:?}, found {:?}. {:?}", kind, tok.kind, tok);
            return Option::None;
        }
        debug!("expected type {:?}, but EOF.", kind);
        return Option::None;
    }

    fn parse_dom_attr(&mut self) -> Option<ast::DomAttr<'a>> {
        if let Some(tok) = self.take() {
            if let TokenKind::DomTagAttrStart = tok.kind {
                let name = unsafe { from_utf8_unchecked(tok.str) };
                debug!("dom_attr name: {}", name);
                let mut node = ast::DomAttr::new(tok);
                if self.skip_symbol(vec![ascii::EQS]) {
                    self.set_breakpoint(Box::new(|owner: &mut Parser| -> bool {
                        if let Option::Some(tok) = owner.take() {
                            match tok.kind {
                                TokenKind::DomTagAttrEnd => return true,
                                _ => {}
                            }
                            owner.back(tok);
                        }
                        return false;
                    }));
                    self.parse_until(&mut node.value);
                    self.pop_breakpoint();
                }
                if let Some(tok) = self.expect_type(TokenKind::DomTagAttrEnd) {
                    return Some(node);
                }
            } else {
                self.back(tok);
            }
        }
        return Option::None;
    }

    fn parse_dom_tag(&mut self, tok: Token<'a>) -> ast::DomNode<'a> {
        let mut tag = ast::DomNode::new(tok);
        while let Some(attr) = self.parse_dom_attr() {
            tag.attrs.push(attr);
        }
        //todo:检查错误
        let mut parse_children = false;
        if let Some(tok) = self.expect_type(TokenKind::DomTagEnd) {
            parse_children = tok.str[0] == ascii::EQS;
        } else {
            //
        }
        if !parse_children {
            return tag;
        }
        self.set_breakpoint(Box::new(|owner: &mut Parser| -> bool {
            let mut tag_name: Vec<u8> = Vec::new();
            for c in tag.pos.str {
                tag_name.push(c.clone());
            }
            let mut breaks: Vec<BreakPoint> = vec![
                BreakPoint::new(false, vec![vec![ascii::SLA], tag_name]),
            ];

            let mut found = true;
            for point in &breaks {
                if point.values.is_empty() {
                    continue;
                }
                found = true;
                let buf: Vec<Token> = vec![];
                for value in &point.values {
                    if let Option::Some(tok) = owner.take() {
                        if !value.compare(tok.str) {
                            found = false;
                        }
                        buf.push(tok);
                    } else {
                        found = false;
                    }
                    if !found {break;}
                }
                if !found || point.keep {
                    while !buf.is_empty() {
                        owner.back(buf.pop().unwrap());
                    }
                }

                if found {
                    return true;
                }
            }
            return false;
        }));
        self.parse_until(&mut tag.children);
        self.pop_breakpoint();
        return tag;
    }

    fn parse_until(&mut self, buf: &mut ast::NodeList<'a>) {
        while !self.check_breakpoint() {
            if let Some(tok) = self.take() {
                match tok.kind {
                    TokenKind::DomTagStart => {
                        buf.push(Node::DomNode(self.parse_dom_tag(tok)));
                    }
                    _ => {}
                }
            } else { break; }
        }
    }

    pub fn parse(&mut self) -> ast::Node<'a> {
        let mut buf: ast::NodeList = vec![];
        self.parse_until(&mut buf);
        ast::Node::None
    }
}

#[test]
fn test_parse() {
    let mut eof = false;
    let mut parser = Parser::new();
    parser.parse();
}