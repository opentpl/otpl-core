use super::ast;
use super::ast::Node;
use super::ast::NodeList;
use super::token::TokenKind;
use super::token::Token;
use super::scanner::Scanner;
use super::token::ascii;
use super::token::Source;

mod breakpoint;

//unimplemented!()
use self::breakpoint::BreakPoint;

pub trait Scanner2 {
    fn back(&mut self, tok: Token);
    fn scan(&mut self) -> Option<Token>;

    fn content(&self, tok: &Token) -> &[u8];
    fn content_vec(&self, tok: &Token) -> Vec<u8>;
}

impl<'a, 'b: 'a> Scanner2 for Scanner<'a, 'b> {
    fn back(&mut self, tok: Token) {
        self.back(tok);
    }

    fn scan(&mut self) -> Option<Token> {
        self.scan()
    }
    fn content(&self, tok: &Token) -> &[u8] {
        self.source.content(tok)
    }
    fn content_vec(&self, tok: &Token) -> Vec<u8> {
        let s = self.source.content(tok);
        let mut arr: Vec<u8> = Vec::new();
        arr.extend_from_slice(s);
        return arr;
    }
}


pub struct Parser<T> {
    scanner: T,
    break_checkers: Vec<Box<(FnMut(&mut Parser<T>) -> bool)>>,
}


impl<T: Scanner2> Parser<T> {
    pub fn new(scanner: T) -> Parser<T> {
        return Parser {
            scanner: scanner,
            break_checkers: vec![],
        };
    }

    fn take(&mut self) -> Option<Token> {
        self.scanner.scan()
    }
    fn back(&mut self, tok: Token) {
        self.scanner.back(tok);
    }

    //    fn src<T:Source>(&self) -> &T{
    //       self.scanner.source.as_ref()
    //    }

    fn skip_symbol(&mut self, symbol: Vec<u8>) -> Option<Token> {
        if let Some(tok) = self.take() {
            if &TokenKind::Symbol == tok.kind() {
                return Some(tok);
            }
            self.back(tok);
        }
        return Option::None;
    }

    fn skip_type(&mut self, kind: TokenKind) -> Option<Token> {
        if let Some(tok) = self.take() {
            if tok.kind() == &kind {
                return Some(tok);
            }
            self.back(tok);
        }
        return Option::None;
    }

    fn set_breakpoint(&mut self, checker: Box<(FnMut(&mut Parser<T>) -> bool)>) {
        self.break_checkers.push(checker);
    }

    fn pop_breakpoint(&mut self) -> Option<Box<(FnMut(&mut Parser<T>) -> bool)>> {
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
    fn expect_type(&mut self, kind: TokenKind) -> Result<Token, String> {
        if let Some(tok) = self.take() {
            if tok.kind() == &kind {
                return Ok(tok);
            }
            return Err(format!("expected type {:?}, found {:?}. {:?}", kind, *tok.kind(), tok));
        }
        return Err(format!("expected type {:?}, but EOF.", kind));
    }

    fn parse_dom_attr(&mut self) -> Option<ast::DomAttr> {
        if let Some(tok) = self.take() {
            if &TokenKind::DomAttrStart != tok.kind() {
                self.back(tok);
                return Option::None;
            }
            let mut node = ast::DomAttr::new(tok);
            if let Some(tok) = self.skip_symbol(vec![ascii::EQS]) {
                self.set_breakpoint(Box::new(|owner: &mut Parser<T>| -> bool {
                    if let Option::Some(tok) = owner.skip_type(TokenKind::DomAttrEnd) {
                        owner.back(tok); // 保留以方便后面检查结束
                        return true;
                    }
                    return false;
                }));
                self.parse_until(&mut node.value);
                self.pop_breakpoint();
            }

            if self.expect_type(TokenKind::DomAttrEnd).is_ok() {
                return Some(node);
            }
        }
        return Option::None;
    }

    fn parse_dom_tag(&mut self, tok: Token) -> Option<ast::DomTag> {
        let mut tag = ast::DomTag::new(tok);
        while let Some(attr) = self.parse_dom_attr() {
            tag.attrs.push(attr);
        }
        //todo: 检查错误
        if let Ok(tok) = self.expect_type(TokenKind::DomTagEnd) {
            // 如果不是独立标签 /
            if self.scanner.content(&tok)[0] == ascii::SLA {
                return Some(tag);
            }
        } else {
            return Option::None;
        }
        let name = self.scanner.content_vec(&tag.name);// 放在这里的原因是因为 所有权移动
        //todo: 考虑，没有按标准(如：html标准dom)来的情况
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(false, TokenKind::DomTagEnd, vec![vec![ascii::SLA], name]),
        ]));
        self.parse_until(&mut tag.children);
        self.pop_breakpoint();
        return Some(tag);
    }


    fn parse_until(&mut self, buf: &mut ast::NodeList) {
        while !self.check_breakpoint() {
            if let Option::Some(tok) = self.take() {
                //debug!("{:?} {:?}", &tok, String::from_utf8_lossy(self.scanner.source.content(&tok)));
                match tok.kind() {
                    &TokenKind::DomTagStart => {
                        if let Some(node) = self.parse_dom_tag(tok) {
                            buf.push(Node::DomTag(node));
                        }
                        break;
                    }
                    &TokenKind::Data => {
                        buf.push(Node::Literal(tok));
                    }
                    _ => {
                        debug!("TODO: no parsing token: {:?}", tok);
                    }
                }
            } else { break; }
        }
    }

    pub fn parse(&mut self) -> ast::Node {
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
    use core::token::*;
    use std::fs::OpenOptions;
    use std::io::prelude::*;
    use core::scanner::SourceReader;
    use std::cell::{Ref, RefCell};


    struct TestVisitor<'a, T: 'a>(&'a T);

    impl<'a, T: Source> Visitor for TestVisitor<'a, T> {
        fn visit_dom_tag(&mut self, tag: &ast::DomTag) {
            println!("tag=> {:?}", tag.name.content_str(self.0));
            for attr in &tag.attrs {
                println!("attr=> {:?}", attr.name.content_str(self.0));
                println!("value=> ");
                self.visit_list(&attr.value)
            }
            println!("children=> ");
            self.visit_list(&tag.children);
            println!("<=tag {:?}", tag.name.content_str(self.0));
        }
        fn visit_literal(&mut self, tok: &Token) {
            debug!("literal=> {:?}", tok.content_str(self.0));
        }
    }

    fn parse(sr: &mut SourceReader) -> ast::Node {
        let mut scanner = Scanner::new(sr);
        let mut parser = Parser::new(scanner);
        return parser.parse();
        // return ast::Node::Empty;
    }

    fn visit<'a, T: Source>(sr: &'a T) {
        let mut visitor = TestVisitor(sr);
    }

    #[test]
    fn test_parse() {
        let mut buf = Vec::new();
        let mut options = OpenOptions::new().read(true).open("./src/core/scanner/test.html");
        match options {
            Err(e) => {
                println!("{}", e);
            }
            Ok(ref mut f) => {
                f.read_to_end(&mut buf);
                println!("打开文件：{:?}", f);
            }
        }
        //

        let mut sr = SourceReader(&buf, "source".as_ref(), 0, vec![]);
        {
            let root = parse(&mut sr);
            let mut visitor = TestVisitor(&sr);
            visitor.visit(&root);
            println!("Parse Done! ==============================");
        }

        {
            //visit(&sr);
            println!("Visit Done! ==============================");
        }
        //end
    }
}
