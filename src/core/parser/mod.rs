use super::ast;
use super::ast::{Node, NodeList};
use super::token::{Token, TokenKind};
use super::scanner::{Scanner};
use super::token::ascii;

mod breakpoint;

pub use self::breakpoint::BreakPoint;

pub struct Parser<'a> {
    scanner: &'a mut Scanner,
    break_checkers: Vec<Box<(FnMut(&mut Parser) -> bool)>>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &mut Scanner) -> Parser {
        return Parser {
            scanner: scanner,
            break_checkers: vec![],
        };
    }

    fn take(&mut self) -> Option<Token> {
        let tok = self.scanner.scan().unwrap();
        match tok.kind() {
            &TokenKind::EOF => {
                return None;
            }
            _ => {}
        }
        Some(tok)
    }
    fn back(&mut self, tok: Token) {
        self.scanner.back(tok);
    }


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
            if self.scanner.source().content(&tok)[0] == ascii::SLA {
                return Some(tag);
            }
        } else {
            return Option::None;
        }
        let name = self.scanner.source().content_vec(&tag.name);// 放在这里的原因是因为 所有权移动
        //todo: 考虑，没有按标准(如：html标准dom)来的情况
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(false, TokenKind::DomTagEnd, vec![vec![ascii::SLA], name]),
        ]));
        self.parse_until(&mut tag.children);
        self.pop_breakpoint();
        return Some(tag);
    }


    fn parse_until(&mut self, buf: &mut NodeList) {
        while !self.check_breakpoint() {
            if let Option::Some(tok) = self.take() {
                //debug!("{:?} {:?}", &tok, self.scanner.source().content(&tok));
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
    use core::scanner::{BytesScanner, Source};
    use core::ast;
    use core::ast::Visitor;
    use core::token::Token;
    use std::fs::OpenOptions;
    use std::io::prelude::*;
    //    use core::scanner::SourceReader;
    //    use std::cell::{Ref, RefCell};


    struct TestVisitor<'a>(&'a Source);

    impl<'a> Visitor for TestVisitor<'a> {
        fn visit_dom_tag(&mut self, tag: &ast::DomTag) {
            println!("tag=> {:?}", self.0.content_str(&tag.name));
            for attr in &tag.attrs {
                println!("attr=> {:?}", self.0.content_str(&attr.name));
                println!("value=> ");
                self.visit_list(&attr.value)
            }
            println!("children=> ");
            self.visit_list(&tag.children);
            println!("<=tag {:?}", self.0.content_str(&tag.name));
        }
        fn visit_literal(&mut self, tok: &Token) {
            debug!("literal=> {:?}", self.0.content_str(tok));
        }
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
                //println!("打开文件：{:?}", f);
            }
        }
        //

        let mut scanner = BytesScanner::new(&buf, "source".as_ref());
        let root: ast::Node;
        {
            let mut parser = Parser::new(&mut scanner);
            root = parser.parse();
            println!("Parse Done! ==============================");
        }

        {
            let mut visitor = TestVisitor(&scanner);
            visitor.visit(&root);
            println!("Visit Done! ==============================");
        }
        //end
    }
}
