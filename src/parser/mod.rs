mod breakpoint;

pub use self::breakpoint::BreakPoint;

use ast;
use ast::{Node, NodeList};
use token::{Token, TokenKind, ascii};
use scanner::{Tokenizer};
use util::{VecSliceCompare};

use {Error, Result, NoneResult};


pub struct Parser<'a> {
    tokenizer: &'a mut Tokenizer,
    break_checkers: Vec<Box<(FnMut(&mut Parser) -> NoneResult)>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: &mut Tokenizer) -> Parser {
        return Parser {
            tokenizer: tokenizer,
            break_checkers: vec![],
        };
    }

    fn take(&mut self) -> Result<Token> {
        self.tokenizer.scan()
    }
    fn back(&mut self, tok: Token) {
        self.tokenizer.back(tok);
    }


    fn skip_symbol(&mut self, symbol: Vec<u8>) -> Result<Token> {
        return self.take().and_then(|tok| -> Result<Token>{
            if &TokenKind::Symbol == tok.kind() {
                let val = self.tokenizer.source().content_vec(&tok);
                if val.compare(symbol.as_ref()) { return Ok(tok); }
            }
            self.back(tok);
            return Err(Error::None);
        });
    }

    fn skip_type(&mut self, kind: TokenKind) -> Option<Token> {
        if let Ok(tok) = self.take() {
            if tok.kind() == &kind {
                return Some(tok);
            }
            self.back(tok);
        }
        return Option::None;
    }

    fn set_breakpoint(&mut self, checker: Box<(FnMut(&mut Parser) -> NoneResult)>) {
        self.break_checkers.push(checker);
    }

    fn pop_breakpoint(&mut self) -> Option<Box<(FnMut(&mut Parser) -> NoneResult)>> {
        self.break_checkers.pop()
    }

    fn check_breakpoint(&mut self) -> NoneResult {
        if self.break_checkers.is_empty() { return Err(Error::None); }
        let mut checker = self.break_checkers.pop().unwrap();
        let result = checker.as_mut()(self);
        self.break_checkers.push(checker);
        return result;
    }

    /// 期望一个类型。如果未找到则产生一个错误。
    fn expect_type(&mut self, kind: TokenKind) -> Result<Token> {
        return self.take().and_then(|tok| -> Result<Token>{
            if tok.kind() == &kind {
                return Ok(tok);
            }
            return Err(Error::Message(format!("expected type {:?}, found {:?}. {:?}", kind, *tok.kind(), tok)));
        });

        // return Err(Error::Message(format!("expected type {:?}, but EOF.", kind)));
    }

    fn parse_dom_attr(&mut self) -> Result<ast::DomAttr> {
        match self.take() {
            Ok(tok) => {
                if &TokenKind::DomAttrStart != tok.kind() {
                    self.back(tok);
                    return Err(Error::None);
                }
                let mut node = ast::DomAttr::new(tok);
                return self.skip_symbol(vec![ascii::EQS])
                    .and_then(|_| -> NoneResult {
                        self.set_breakpoint(Box::new(|parser: &mut Parser| -> NoneResult {
                            if let Option::Some(tok) = parser.skip_type(TokenKind::DomAttrEnd) {
                                parser.back(tok); // 保留以方便后面检查结束
                                return Error::ok();
                            }
                            return Err(Error::None);
                        }));
                        let rst = self.parse_until(&mut node.value);
                        // println!("4=>>>>>>>>>>>>{:?}", node);
                        self.pop_breakpoint();
                        return rst;
                    })
                    .and_then(|_| self.expect_type(TokenKind::DomAttrEnd))
                    .and_then(|_| Ok(node));
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    fn parse_dom_tag(&mut self, tok: Token) -> Result<ast::DomTag> {
        let mut tag = ast::DomTag::new(tok);
        loop {
            match self.parse_dom_attr() {
                Ok(attr) => {
                    println!("0=>>>>>>>>>>>>{:?}", attr);
                    tag.attrs.push(attr);
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }

        match self.expect_type(TokenKind::DomTagEnd) {
            Ok(tok) => {
                // 如果是独立标签 /
                if self.tokenizer.source().content(&tok)[0] == ascii::SLA {
                    return Ok(tag);
                }
            }
            Err(Error::None) => { return Err(Error::None); }
            Err(err) => { return Err(err); }
        }
        let name = self.tokenizer.source().content_vec(&tag.name);
        //todo: 考虑，没有按标准(如：html标准dom)来的情况
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(false, TokenKind::DomCTag, vec![vec![ascii::SLA], name]),
        ]));
        self.parse_until(&mut tag.children);
        self.pop_breakpoint();
        return Ok(tag);
    }

    fn parse(&mut self) -> Result<ast::Node> {
        return self.take().and_then(|tok| -> Result<ast::Node>{
            match tok.kind() {
                &TokenKind::DomTagStart => {
                    return self.parse_dom_tag(tok).and_then(|node| -> Result<ast::Node>{
                        return Ok(Node::DomTag(node));
                    });
                }
                &TokenKind::Data => {
                    return Ok(Node::Literal(tok));
                }
                _ => {
                    println!("TODO: no parsing token: {:?}", tok);
                    return Err(Error::None);
                }
            }
        });
    }

    fn parse_until(&mut self, buf: &mut NodeList) -> NoneResult {
        loop {
            match self.check_breakpoint() {
                Ok(_) | Err(Error::EOF) => { break; }
                Err(Error::None) => {}
                err => { return err; }
            }

            match self.parse() {
                Ok(node) => { buf.push(node) }
                Err(Error::None) | Err(Error::EOF) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Error::ok();
    }

    fn parse_all(&mut self, buf: &mut NodeList) -> NoneResult {
        loop {
            match self.parse() {
                Ok(node) => { buf.push(node) }
                Err(Error::None) | Err(Error::EOF) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Error::ok();
    }

    pub fn parse_root(&mut self) -> ast::Node {
        let mut root = ast::Root::new();
        self.parse_all(&mut root.body).expect("Failed to parse");
        return Node::Root(root);
    }
}
