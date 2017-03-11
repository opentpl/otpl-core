use super::ast;
use super::ast::Node;
use super::ast::NodeList;
use super::token::TokenKind;
use super::token::Token;
use super::scanner::Scanner;
use super::token::ascii;
//use std::ops::Index;
//use std::str::from_utf8_unchecked;
//use std::cell::RefCell;
//use std::sync::Arc;

//use util::VecSliceCompare;
//use util::Queue;

mod breakpoint;

use self::breakpoint::BreakPoint;


pub struct Parser<'a> {
    scanner: Scanner<'a>,
    token_buf: Vec<Token<'a>>,
    breaks: Vec<BreakPoint>,
    break_checkers: Vec<Box<(FnMut(&mut Parser) -> bool)>>,
}


impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Parser<'a> {
        //        let mut scanner = Scanner::new("<div id=\"te\\\"st\">".as_bytes(), "{{".as_bytes(), "}}".as_bytes());
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

    fn skip_symbol(&mut self, symbol: Vec<u8>) -> Option<Token<'a>> {
        if let Some(tok) = self.take() {
            if TokenKind::Symbol == tok.kind {
                return Some(tok);
            }
            self.back(tok);
        }
        return Option::None;
    }

    fn skip_type(&mut self, kind: TokenKind) -> Option<Token<'a>> {
        if let Some(tok) = self.take() {
            if tok.kind == kind {
                return Some(tok);
            }
            self.back(tok);
        }
        return Option::None;
    }

    fn set_breakpoint(&mut self, checker: Box<(FnMut(&mut Parser) -> bool)>) {
        self.break_checkers.push(checker);
    }

    fn pop_breakpoint(&mut self) -> Option<Box<(FnMut(&mut Parser) -> bool)>> {
        self.break_checkers.pop()
    }

    fn check_breakpoint(&mut self) -> bool {
        if self.break_checkers.is_empty() {
            return false;
        }
        let mut checker = self.break_checkers.pop().unwrap();
        let result = checker.as_mut()(self);
        self.break_checkers.push(checker);
        return result;
    }

    /// 期望一个类型。如果未找到则产生一个错误。
    fn expect_type(&mut self, kind: TokenKind) -> Option<Token<'a>> {
        if let Some(tok) = self.take() {
            if tok.kind == kind {
                return Some(tok);
            }
            panic!("expected type {:?}, found {:?}. {:?}", kind, tok.kind, tok);
            return Option::None;
        }
        panic!("expected type {:?}, but EOF.", kind);
        return Option::None;
    }

    fn parse_dom_attr(&mut self) -> Option<ast::DomAttr<'a>> {
        if let Some(tok) = self.take() {
            if TokenKind::DomAttrStart != tok.kind {
                self.back(tok);
                return Option::None;
            }
            let mut node = ast::DomAttr::new(tok);
            if let Some(tok) = self.skip_symbol(vec![ascii::EQS]) {
                self.set_breakpoint(Box::new(|owner: &mut Parser| -> bool {
                    if let Option::Some(tok) = owner.skip_type(TokenKind::DomAttrEnd) {
                        owner.back(tok); // 保留以方便后面检查结束
                        return true;
                    }
                    return false;
                }));
                self.parse_until(&mut node.value);
                self.pop_breakpoint();
            }
            if let Some(tok) = self.expect_type(TokenKind::DomAttrEnd) {
                return Some(node);
            }
        }
        return Option::None;
    }

    fn parse_dom_tag(&mut self, tok: Token<'a>) -> Option<ast::DomTag<'a>> {
        let mut tag = ast::DomTag::new(tok);
        while let Some(attr) = self.parse_dom_attr() {
            tag.attrs.push(attr);
        }
        //todo: 检查错误
        let mut parse_children = false;
        if let Some(tok) = self.expect_type(TokenKind::DomTagEnd) {
            parse_children = tok.str[0] != ascii::SLA; // 如果不是独立标签 /
        } else {
            return Option::None;
        }
        if !parse_children {
            return Some(tag);
        }
        //todo: 考虑，没有按标准来的情况
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(false, vec![vec![ascii::SLA], tag.name.src_vec()]),
        ]));
        self.parse_until(&mut tag.children);
        self.pop_breakpoint();
        return Some(tag);
    }

    fn parse_until(&mut self, buf: &mut ast::NodeList<'a>) {
        while !self.check_breakpoint() {
            if let Some(tok) = self.take() {
                match tok.kind {
                    TokenKind::DomTagStart => {
                        if let Some(node) = self.parse_dom_tag(tok) {
                            buf.push(Node::DomTag(node));
                        }
                    }
                    TokenKind::Data => {
                        buf.push(Node::Literal(tok));
                    }
                    _ => {
                        debug!("no parsing token: {:?}", tok);
                    }
                }
            } else { break; }
        }
    }

    pub fn parse(&mut self) -> ast::Node<'a> {
        let mut root = ast::Root::new();
        self.parse_until(&mut root.body);
        return Node::Root(root);
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use core::scanner::Scanner;
    use core::ast;
    use core::ast::Visitor;
    use core::token::Token;
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    struct TestVisitor;

    impl<'a> Visitor<'a> for TestVisitor {
        fn visit_dom_tag(&mut self, tag: &'a ast::DomTag) {
            println!("tag=> {:?}", tag.name.src_str());
            for attr in &tag.attrs {
                println!("attr=> {:?}", attr.name.src_str());
                println!("value=> ");
                self.visit_list(&attr.value)
            }
            self.visit_list(&tag.children);
        }
        fn visit_literal(&mut self, tok: &'a Token) {
            debug!("literal=> {:?}", tok.src_str());
        }
    }

    #[test]
    fn test_parse() {
        let mut options = OpenOptions::new().read(true).open("./src/core/scanner/test.html");
        match options {
            Err(e) => {
                println!("{}", e);
            }
            Ok(ref mut f) => {
                let mut buf = Vec::new();
                f.read_to_end(&mut buf);
                println!("{:?}", f);
                let mut scanner = Scanner::new(&buf,"test.html".as_ref(), "{{".as_bytes(), "}}".as_bytes());
                let mut parser = Parser::new(scanner);
                let root = parser.parse();
                println!("Parse Done! ==============================");
                let mut visitor = TestVisitor;
                visitor.visit(&root);
                println!("Visit Done! ==============================");
            }
        }
    }
}
