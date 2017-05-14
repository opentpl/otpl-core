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


    fn skip_symbol(&mut self, symbols: Vec<Vec<u8>>) -> Result<Token> {
        return self.take().and_then(|tok| -> Result<Token>{
            if &TokenKind::Symbol == tok.kind() {
                let val = self.tokenizer.source().content(&tok);
                for symbol in &symbols {
                    if symbol.compare(val) { return Ok(tok); }
                }
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
            return Err(Error::Message(format!("expected type {:?}, found {:?}", kind, tok)));
        });

        // return Err(Error::Message(format!("expected type {:?}, but EOF.", kind)));
    }

    fn expect_value(&mut self, value: Vec<u8>) -> NoneResult {
        return self.take().and_then(|tok| -> NoneResult{
            let content = self.tokenizer.source().content(&tok);
            if value.compare(content) {
                return Error::ok();
            }
            return Err(Error::Message(format!("expected value {:?}, found {:?}. {:?}", value, content, tok)));
        }).map_err(|err| -> Error{
            match err {
                Error::None => {
                    return Error::Message(format!("expect_value: 代码未结束 {:?}", err))
                }
                _ => { return err; }
            }
        });
    }

    /// 解析DOM标签属性
    fn parse_dom_attr(&mut self) -> Result<ast::DomAttr> {
        match self.take() {
            Ok(tok) => {
                if &TokenKind::DomAttrStart != tok.kind() {
                    self.back(tok);
                    return Err(Error::None);
                }
                let mut node = ast::DomAttr::new(tok);
                return self.skip_symbol(vec![vec![ascii::EQS]])
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
    /// 解析DOM标签
    fn parse_dom_tag(&mut self, tok: Token) -> Result<ast::DomTag> {
        let mut tag = ast::DomTag::new(tok);
        loop {
            match self.parse_dom_attr() {
                Ok(attr) => {
                    //println!("0=>>>>>>>>>>>>{:?}", attr);
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
            Err(Error::None) => { return Err(Error::None); } //TODO:重新定义错误：标签未结束
            Err(err) => { return Err(err); }
        }
        let name = self.tokenizer.source().content_vec(&tag.name);
        //println!("bbbbbbbbbb:{:?}", String::from_utf8(name.clone()).unwrap());
        //todo: 考虑，没有按标准(如：html标准dom)来的情况
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(false, TokenKind::DomCTag, vec![name]),
        ]));

        match self.parse_until(&mut tag.children) {
            Ok(_) => {
                //println!("vvvvvvvvvvvvvvv");
            }
            Err(Error::None) => {
                //                let tok=self.take().unwrap();
                //                println!("xxxxxxxxxxx:{:?}",self.tokenizer.source().content_str(&tok));

                //println!("xxxxxxxxxxxxxxx");
            }
            Err(err) => { return Err(err); }
        }
        self.pop_breakpoint();

        //        if tag.children.len() > 0 {
        //            //移除所匹配到的ctag
        //            let index = tag.children.len() - 1;
        //            tag.children.remove(index);
        //        }

        return Ok(tag);
    }
    /// 解析表达式的独立主体部分
    fn parse_primary(&mut self) -> Result<ast::Node> {
        println!("parse_primary");
        return self.take().and_then(|tok| -> Result<ast::Node>{
            match tok.kind() {
                &TokenKind::Identifier => {
                    let val = self.tokenizer.source().content(&tok);
                    //false|true
                    if vec!['f' as u8, 'a' as u8, 'l' as u8, 's' as u8, 'e' as u8, ].compare(val)
                        || vec!['t' as u8, 'r' as u8, 'u' as u8, 'e' as u8, ].compare(val) {
                        return Ok(Node::Boolean(tok));
                    }
                    //null
                    if vec!['n' as u8, 'u' as u8, 'l' as u8, 'l' as u8].compare(val) {
                        return Ok(Node::None(tok));
                    }
                    println!("Identifier:bbbbbbbbbbbbbbbbb");
                    return Ok(Node::Identifier(tok));
                }
                &TokenKind::Int => {
                    return match self.skip_symbol(vec![vec!['.' as u8]]).and_then(|_| -> Result<Token> { self.expect_type(TokenKind::Int) }) {
                        Ok(precision) => { return Ok(Node::Float(tok, precision)); }
                        Err(Error::None) => {
                            println!("Identifier:int");
                            return Ok(Node::Integer(tok));
                        }
                        Err(err) => { return Err(err); }
                    };
                }
                &TokenKind::Symbol if vec!['(' as u8].compare(self.tokenizer.source().content(&tok)) => {
                    match self.parse_group(vec![')' as u8]) {
                        Ok(list) => { return Ok(Node::List(list)); }
                        Err(err) => { return Err(err); }
                    }
                }
                _ => { return Err(Error::Message(format!("parse_primary: unexpected token {:?}", tok))); }
            }
        }).map_err(|err| -> Error{
            match err {
                Error::None => {
                    return Error::Message(format!("parse_primary: 代码未结束 {:?}", err))
                }
                _ => { return err; }
            }
        });
    }
    /// 解析成员访问
    fn parse_member_access(&mut self) -> Result<ast::Node> {
        println!("parse_member_access");
        let node = self.parse_primary();
        if node.is_err() { return node; }
        let mut node = node.unwrap();
        let symbols = vec![vec!['.' as u8], vec!['[' as u8], vec!['(' as u8]];
        loop {
            match self.skip_symbol(symbols.clone()) {
                Ok(operator) => {
                    if symbols[0].compare(self.tokenizer.source().content(&operator)) {
                        match self.expect_type(TokenKind::Identifier) {
                            Ok(tok) => {
                                node = Node::Property(Box::new(node), vec![Node::String(tok)], operator);
                            }
                            Err(err) => { return Err(err); }
                        }
                    } else if symbols[1].compare(self.tokenizer.source().content(&operator)) {
                        match self.parse_group(vec![']' as u8]) {
                            Ok(list) => {
                                node = Node::Property(Box::new(node), list, operator);
                            }
                            Err(err) => { return Err(err); }
                        }
                    } else if symbols[2].compare(self.tokenizer.source().content(&operator)) {
                        match self.parse_group(vec![')' as u8]) {
                            Ok(list) => {
                                node = Node::Method(Box::new(node), list, operator);
                            }
                            Err(err) => { return Err(err); }
                        }
                    }
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Ok(node);
    }
    /// 解析一元运算
    fn parse_unary(&mut self) -> Result<ast::Node> {
        println!("parse_unary");
        match self.skip_symbol(vec![vec!['-' as u8], vec!['+' as u8]]) {
            Ok(operator) => {
                //TODO: - = neg, + = pos
                let node = self.parse_member_access();
                if node.is_err() { return node; }
                return Ok(Node::Unary(Box::new(node.unwrap()), operator));
            }
            Err(Error::None) => {}
            Err(err) => {
                println!("parse_unary:err:{:?}", err);
                return Err(err);
            }
        }
        return self.parse_member_access();
    }
    /// 解析乘除运算
    fn parse_binary_mdm(&mut self) -> Result<ast::Node> {
        println!("parse_binary_mdm");
        let node = self.parse_unary();
        if node.is_err() { return node; }
        let mut node = node.unwrap();
        loop {
            match self.skip_symbol(vec![vec!['*' as u8], vec!['/' as u8], vec!['%' as u8]]) {
                Ok(operator) => {
                    let right = self.parse_unary();
                    if right.is_err() { return right; }
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), operator);
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Ok(node);
    }
    /// 解析加减运算
    fn parse_binary_as(&mut self) -> Result<ast::Node> {
        println!("parse_binary_as");
        let node = self.parse_binary_mdm();
        if node.is_err() { return node; }
        let mut node = node.unwrap();
        loop {
            match self.skip_symbol(vec![vec!['+' as u8], vec!['-' as u8]]) {
                Ok(operator) => {
                    let right = self.parse_binary_mdm();
                    if right.is_err() { return right; }
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), operator);
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Ok(node);
    }
    /// 解析比较运算
    fn parse_compare(&mut self) -> Result<ast::Node> {
        println!("parse_compare");
        let node = self.parse_binary_as();
        if node.is_err() { return node; }
        let mut node = node.unwrap();
        loop {
            match self.skip_symbol(vec![vec!['=' as u8, '=' as u8]
                                        , vec!['!' as u8, '=' as u8]
                                        , vec!['<' as u8, '=' as u8]
                                        , vec!['>' as u8, '=' as u8]
                                        , vec!['<' as u8]
                                        , vec!['>' as u8]]) {
                Ok(operator) => {
                    let right = self.parse_binary_as();
                    if right.is_err() { return right; }
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), operator);
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Ok(node);
    }
    /// 解析逻辑运算
    fn parse_logic(&mut self) -> Result<ast::Node> {
        println!("parse_logic");
        let node = self.parse_compare();
        if node.is_err() { return node; }
        let mut node = node.unwrap();
        loop {
            match self.skip_symbol(vec![vec!['?' as u8, '?' as u8], vec!['|' as u8, '|' as u8], vec!['&' as u8, '&' as u8]]) {
                Ok(operator) => {
                    let right = self.parse_compare();
                    if right.is_err() { return right; }
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), operator);
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Ok(node);
    }
    /// 解析三目运算
    fn parse_ternary(&mut self) -> Result<ast::Node> {
        println!("parse_ternary");
        let node = self.parse_logic();
        if node.is_err() { return node; }
        let mut node = node.unwrap();
        loop {
            match self.skip_symbol(vec![vec!['?' as u8]]) {
                Ok(_) => {
                    let left = self.parse_expression();
                    if left.is_err() { return left; }
                    match self.expect_value(vec![':' as u8]) {
                        Ok(_) => {}
                        Err(err) => { return Err(err); }
                    }
                    let right = self.parse_expression();
                    if right.is_err() { return right; }
                    node = Node::Ternary(Box::new(node), Box::new(left.unwrap()), Box::new(right.unwrap()));
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        return Ok(node);
    }
    /// 解析一个表达式
    fn parse_expression(&mut self) -> Result<ast::Node> {
        self.parse_ternary()
    }
    /// 解析一个组
    fn parse_group(&mut self, end: Vec<u8>) -> Result<NodeList> {
        println!("parse_group");
        let mut list = vec![];
        match self.skip_symbol(vec![end]) {
            Ok(_) => { return Ok(list); }
            Err(err) => { return Err(err); }
        }
        loop {
            match self.parse_expression() {
                Ok(node) => {
                    list.push(node);
                }
                Err(Error::None) => { return Err(Error::Message("expected 表达式".to_string())); }
                Err(err) => { return Err(err); }
            }
            match self.skip_symbol(vec![end.clone(), vec![',' as u8]]) {
                Ok(tok) => {
                    let val = self.tokenizer.source().content(&tok);
                    if end.compare(val) {
                        return Ok(list);
                    }
                }
                Err(Error::None) => { return Err(Error::Message("expected '".to_string())); }
                Err(err) => { return Err(err); }
            }
        }
    }
    fn parse_dict() {}
    fn parse_else(&mut self, key: Vec<u8>) -> Result<ast::Node> {
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['/' as u8], key]),
        ]));
        let mut body = vec![];
        match self.parse_until(&mut body) {
            Ok(_) => {}
            Err(Error::None) => { return Err(Error::Message("XX 命令未结束：语法/xx".to_string())); }
            Err(err) => { return Err(err); }
        }
        self.pop_breakpoint();
        //跳过边界
        match self.expect_type(TokenKind::RDelimiter) {
            Ok(_) => {}
            Err(err) => { return Err(err); }
        }
        return Ok(Node::Else(body));
    }
    fn parse_if(&mut self) -> Result<ast::Node> {
        println!("parse_if");
        let condition = self.parse_expression();
        if condition.is_err() { return condition; }
        //跳过边界
        match self.expect_type(TokenKind::RDelimiter) {
            Ok(_) => {}
            Err(err) => {
                println!("zzzzzzzzz");
                return Err(err);
            }
        }
        println!("xxxxxxxxxxxxxxxxxxxxx");
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['{' as u8, '{' as u8, ], vec!['e' as u8, 'l' as u8, 'i' as u8, 'f' as u8, ]]),
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['{' as u8, '{' as u8, ], vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ]]),
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['{' as u8, '{' as u8, ], vec!['/' as u8], vec!['i' as u8, 'f' as u8, ]]),
        ]));
        let mut body = vec![];
        match self.parse_until(&mut body) {
            Ok(_) => {}
            Err(Error::None) => { return Err(Error::Message("if 命令未结束：必须至少包含 elif 或 else 或 /if其中之一".to_string())); }
            Err(err) => { return Err(err); }
        }
        self.pop_breakpoint();
        let mut items = vec![];
        loop {
            match self.skip_type(TokenKind::LDelimiter).and_then(|tok| -> Option<Token>{
                return self.skip_type(TokenKind::Identifier).or_else(|| -> Option<Token>{
                    self.back(tok);
                    return None;
                });
            }).ok_or(Error::None).and_then(|tok| -> Result<ast::Node> {
                //elif
                if vec!['e' as u8, 'l' as u8, 'i' as u8, 'f' as u8, ]
                    .compare(self.tokenizer.source().content(&tok)) {
                    return self.parse_if();
                }
                //else
                if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ]
                    .compare(self.tokenizer.source().content(&tok)) {
                    return self.parse_else(vec!['i' as u8, 'f' as u8, ]);
                }
                self.back(tok);
                return Err(Error::None);
            }) {
                Ok(node) => {
                    items.push(node);
                }
                Err(Error::None) => { break; }
                err => { return err; }
            }
        }

        match self.expect_type(TokenKind::LDelimiter)
            .and_then(|_| -> NoneResult{ self.expect_value(vec!['/' as u8]) })
            .and_then(|_| -> NoneResult{ self.expect_value(vec!['i' as u8, 'f' as u8, ]) }) {
            Ok(_) => { return Ok(Node::If(Box::new(condition.unwrap()), body, items)); }
            Err(Error::None) => { return Err(Error::Message("if 命令未结束：必须以/if结束".to_string())); }
            Err(err) => { return Err(err); }
        }
    }

    /// 解析代码段
    fn parse_statement(&mut self) -> Result<ast::Node> {
        println!("parse_statement");
        let mut list = vec![];
        loop {
            match self.take().and_then(|tok| -> Result<ast::Node>{
                return match tok.kind() {
                    &TokenKind::RDelimiter => { return Err(Error::None); }
                    &TokenKind::Identifier => {
                        println!("yyyyyyyyyyyyyyyyyyyy");
                        //if
                        if vec!['i' as u8, 'f' as u8, ].compare(self.tokenizer.source().content(&tok)) {
                            return self.parse_if();
                        }
                        println!("zzzzzzzzzzzzzzzzzzzzz");
                        return self.parse_expression();
                    }
                    _ => {
                        return Err(Error::Message(format!("parse_statement: unexpected token {:?}", tok)));
                    }
                };
            }) {
                Ok(node) => {
                    list.push(node);
                }
                Err(Error::None) => { return Ok(Node::Statement(list)); }
                err => { return err; }
            }
        }
    }

    fn parse(&mut self) -> Result<ast::Node> {
        return self.take().and_then(|tok| -> Result<ast::Node>{
            match tok.kind() {
                &TokenKind::DomTagStart => {
                    return self.parse_dom_tag(tok).and_then(|node| -> Result<ast::Node>{
                        return Ok(Node::DomTag(node));
                    });
                }
                &TokenKind::LDelimiter => {
                    println!("ffffffffffffff");
                    return self.parse_statement();
                }
                &TokenKind::Data => {
                    return Ok(Node::Literal(tok));
                }
                _ => {
                    println!("TODO: no parsing token: {:?}", tok);
                    return Ok(Node::Empty);
                }
            }
        });
    }

    fn parse_until(&mut self, buf: &mut NodeList) -> NoneResult {
        // TODO: 设置还原点
        self.tokenizer.mark();
        loop {
            match self.check_breakpoint() {
                //println!("zzzzzzzzzzzzz");
                Ok(_) => {
                    self.tokenizer.unmark();
                    return Error::ok();
                }
                Err(Error::EOF) => { break; }
                Err(Error::None) => {}
                err => { return err; }
            }

            match self.parse() {
                Ok(node) => { buf.push(node) }
                Err(Error::None) | Err(Error::EOF) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        // TODO: 还原点
        self.tokenizer.reset();
        buf.clear();
        //println!("fffffffffffff");
        return Err(Error::None);
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
