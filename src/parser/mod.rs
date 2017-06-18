mod breakpoint;

pub use self::breakpoint::BreakPoint;

use ast;
use ast::{Node, NodeList};
use token::{Token, TokenKind, ascii};
use scanner::{Tokenizer};
use util::{VecSliceCompare};
use scanner::BytesScanner;
use {Error, Result, NoneResult};
use std::str::from_utf8_unchecked;

fn optimize_literal(value: &[u8]) -> (usize, usize) {
    let mut start = 0usize;
    let mut end = value.len();
    for i in 0..value.len() {
        let ch = value[i];
        if !(ch == ('\r' as u8) || ch == ('\n' as u8) || ch == ('\t' as u8) || ch == (' ' as u8)) {
            start = i;
            break;
        }
    }
    for i in (0..value.len()).rev() {
        let ch = value[i];
        if !(ch == ('\r' as u8) || ch == ('\n' as u8) || ch == ('\t' as u8) || ch == (' ' as u8)) {
            end = i + 1;
            break;
        }
        end = i;
    }
    println!("abc:{} {} {}", start, end, value.len());
    return (start, end);
}

fn get_operator(operator: Token) -> ast::Operator {
    if vec!['+' as u8, ].compare(operator.value()) {
        return ast::Operator::Add;
    } else if vec!['-' as u8, ].compare(operator.value()) {
        return ast::Operator::Sub;
    } else if vec!['*' as u8, ].compare(operator.value()) {
        return ast::Operator::Mul;
    } else if vec!['/' as u8, ].compare(operator.value()) {
        return ast::Operator::Div;
    } else if vec!['%' as u8, ].compare(operator.value()) {
        return ast::Operator::Mod;
    } else if vec!['<' as u8, ].compare(operator.value()) {
        return ast::Operator::Lt;
    } else if vec!['>' as u8, ].compare(operator.value()) {
        return ast::Operator::Gt;
    } else if vec!['=' as u8, '=' as u8, ].compare(operator.value()) {
        return ast::Operator::Eq;
    } else if vec!['!' as u8, '=' as u8, ].compare(operator.value()) {
        return ast::Operator::NotEq;
    } else if vec!['<' as u8, '=' as u8, ].compare(operator.value()) {
        return ast::Operator::Lte;
    } else if vec!['>' as u8, '=' as u8, ].compare(operator.value()) {
        return ast::Operator::Gte;
    } else if vec!['&' as u8, '&' as u8, ].compare(operator.value()) {
        return ast::Operator::And;
    } else if vec!['|' as u8, '|' as u8, ].compare(operator.value()) {
        return ast::Operator::Or;
    } else if vec!['?' as u8, '?' as u8, ].compare(operator.value()) {
        return ast::Operator::NullCond;
    }
    panic!("undefined operator: {:?}", operator);
}

fn err(dev_prefix: &str, msg: String, offs: usize) -> Error {
    Error::Parse(format!("{}:{}", dev_prefix, msg), offs)
}

fn vec_str(value: &Vec<u8>) -> &str {
    unsafe { from_utf8_unchecked(value.as_slice()) }
}

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
        self.tokenizer.back_token(tok);
    }


    fn skip_symbol(&mut self, symbols: Vec<Vec<u8>>) -> Result<Token> {
        return self.take().and_then(|tok| -> Result<Token>{
            if &TokenKind::Symbol == tok.kind() {
                for symbol in &symbols {
                    if symbol.compare(tok.value()) { return Ok(tok); }
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
            return Err(err("expect_type", format!("expected type {:?}, found {:?}", kind, tok.kind()), tok.offset()));
        });

        // return Err(Error::Message(format!("expected type {:?}, but EOF.", kind)));
    }

    fn expect_value(&mut self, value: Vec<u8>) -> NoneResult {
        return self.take().and_then(|tok| -> NoneResult{
            if value.compare(tok.value()) {
                return Error::ok();
            }
            return Err(err("expect_value", format!("expected value {:?}, found {:?}", vec_str(&value), tok.value_str()), tok.offset()));
        }).map_err(|e| -> Error{
            match e {
                Error::None => {
                    //TODO: 具体错误原因
                    return err("expect_value", format!("expected value {:?}, but EOF?", vec_str(&value)), 0);
                }
                _ => { return e; }
            }
        });
    }


    /// 解析DOM标签属性
    fn parse_dom_attr(&mut self) -> Result<ast::DomAttr> {
        println!("parse_dom_attr");
        match self.take() {
            Ok(tok) => {
                if &TokenKind::DomAttrStart != tok.kind() {
                    self.back(tok);
                    return Err(Error::None);
                }
                let mut node = ast::DomAttr::new(tok.clone());
                return self.expect_type(TokenKind::DomAttrValue).and_then(|attr_val| -> NoneResult{
                    let val = attr_val.value();
                    let name = tok.value();
                    let pos = attr_val.1;
                    let mut value: Result<NodeList>;
                    if name[0] == '@' as u8 {
                        let mut name = &name[1..name.len()];
                        if vec!['i' as u8, 'f' as u8, ].compare(name) {} else if vec!['f' as u8, 'o' as u8, 'r' as u8, ].compare(name) {} else if vec!['e' as u8, 'l' as u8, 'i' as u8, 'f' as u8, ].compare(name) {
                            name = &name[2..name.len()];
                        } else if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ].compare(name) {
                            // else 不解析值
                            println!("else不解析值");
                            return Error::ok();
                        } else {
                            return Err(err("parse_dom_attr", format!("Unsupported extends command: {:?}", unsafe { from_utf8_unchecked(name) }), tok.offset()));
                        }
                        let start = name.len() + 3;
                        let mut s = String::from("{{");
                        s += unsafe { from_utf8_unchecked(name) };
                        s += " ";
                        s += unsafe { from_utf8_unchecked(val) };
                        s += "}}";
                        s += "{{/";
                        s += unsafe { from_utf8_unchecked(name) };
                        s += "}}";
                        println!("K=>>>>>>>>>> {:?}", s);
                        let mut inner = BytesScanner::new(s.as_bytes(), "inner-ext".as_ref());
                        // 重新定位
                        let mut buf = vec![];
                        loop {
                            match inner.scan() {
                                Ok(mut tok) => {
                                    tok.1 += pos - start;
                                    buf.push(tok);
                                }
                                Err(Error::EOF) => { break; }
                                Err(err) => { return Err(err); }
                            }
                        }
                        while !buf.is_empty() {
                            inner.back_token(buf.pop().unwrap());
                        }
                        value = Parser::new(&mut inner).parse_all();
                    } else {
                        let mut inner = BytesScanner::new(val, "inner-attr".as_ref());
                        let mut buf = vec![];
                        loop {
                            match inner.scan() {
                                Ok(mut tok) => {
                                    tok.1 += pos - 1;
                                    buf.push(tok);
                                }
                                Err(Error::EOF) => { break; }
                                Err(err) => { return Err(err); }
                            }
                        }
                        while !buf.is_empty() {
                            inner.back_token(buf.pop().unwrap());
                        }
                        value = Parser::new(&mut inner).parse_all();
                    }
                    //println!("999999999999999999999:{:?}", value);
                    match value {
                        Ok(mut list) => {
                            node.value.append(&mut list);
                        }
                        //TODO: 重写错误定位
                        Err(err) => { return Err(err); }
                    }
                    return Error::ok();
                }).and_then(|_| self.expect_type(TokenKind::DomAttrEnd)).and_then(|_| Ok(node));
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    /// 解析DOM标签
    fn parse_dom_tag(&mut self, tag: Token) -> Result<ast::Node> {
        println!("parse_dom_tag");
        let mut attrs = vec![];
        let mut children = vec![];
        loop {
            match self.parse_dom_attr() {
                Ok(attr) => {
                    //println!("0=>>>>>>>>>>>>{:?}", attr);
                    attrs.push(attr);
                }
                Err(Error::None) => { break; }
                Err(err) => { return Err(err); }
            }
        }

        match self.expect_type(TokenKind::DomTagEnd) {
            Ok(tok) => {
                // 如果是独立标签 /
                if tok.2[0] == ascii::SLA {
                    return Ok(Node::DomTag(tag, attrs, children));
                }
            }
            Err(Error::None) => { return Err(Error::None); } //TODO:重新定义错误：标签未结束
            Err(err) => { return Err(err); }
        }
        let name = tag.2.clone();
        //println!("bbbbbbbbbb:{:?}", String::from_utf8(name.clone()).unwrap());
        //todo: 考虑，没有按标准(如：html标准dom)来的情况
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(false, TokenKind::DomCTag, vec![name]),
        ]));

        match self.parse_until(&mut children) {
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

        return Ok(Node::DomTag(tag, attrs, children));
    }
    /// 解析表达式的独立主体部分
    fn parse_primary(&mut self) -> Result<ast::Node> {
        println!("parse_primary");
        return self.take().and_then(|tok| -> Result<ast::Node>{
            match tok.kind() {
                &TokenKind::Identifier => {
                    //false
                    if vec!['f' as u8, 'a' as u8, 'l' as u8, 's' as u8, 'e' as u8].compare(tok.value()) {
                        return Ok(Node::Const(ast::Constant::False));
                    }
                    //true
                    if vec!['t' as u8, 'r' as u8, 'u' as u8, 'e' as u8].compare(tok.value()) {
                        return Ok(Node::Const(ast::Constant::True));
                    }
                    //null
                    if vec!['n' as u8, 'u' as u8, 'l' as u8, 'l' as u8].compare(tok.value()) {
                        return Ok(Node::Const(ast::Constant::None));
                    }
                    //break
                    if vec!['b' as u8, 'r' as u8, 'e' as u8, 'a' as u8, 'k' as u8].compare(tok.value()) {
                        return Ok(Node::Const(ast::Constant::Break));
                    }
                    //continue
                    if vec!['c' as u8, 'o' as u8, 'n' as u8, 't' as u8, 'i' as u8, 'n' as u8, 'u' as u8, 'e' as u8].compare(tok.value()) {
                        return Ok(Node::Const(ast::Constant::Continue));
                    }
                    println!("Identifier:bbbbbbbbbbbbbbbbb");
                    return Ok(Node::Identifier(tok));
                }
                &TokenKind::Int => {
                    return match self.skip_symbol(vec![vec!['.' as u8]]).and_then(|_| -> Result<Token> { self.expect_type(TokenKind::Int) }) {
                        Ok(precision) => { return Ok(Node::Const(ast::Constant::Float(tok, precision))); }
                        Err(Error::None) => {
                            println!("Identifier:int");
                            return Ok(Node::Const(ast::Constant::Integer(tok)));
                        }
                        Err(err) => { return Err(err); }
                    };
                }
                &TokenKind::Symbol if vec!['(' as u8].compare(tok.value()) => {
                    match self.parse_group(vec![')' as u8]) {
                        Ok(list) => { return Ok(Node::List(list)); }
                        Err(err) => { return Err(err); }
                    }
                }
                _ => {
                    return Err(err("parse_primary", format!("unexpected token: {:?}", tok.value_str()), tok.offset()));
                }
            }
        }).map_err(|e| -> Error{
            match e {
                Error::None => {
                    //TODO: 具体错误原因
                    return err("parse_primary", format!("expected token, but EOF?"), 0);
                }
                _ => { return e; }
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
                    if symbols[0].compare(operator.value()) {
                        match self.expect_type(TokenKind::Identifier) {
                            Ok(tok) => {
                                node = Node::Property(Box::new(node), vec![Node::Const(ast::Constant::String(tok))], operator);
                            }
                            Err(err) => { return Err(err); }
                        }
                    } else if symbols[1].compare(operator.value()) {
                        match self.parse_group(vec![']' as u8]) {
                            Ok(list) => {
                                node = Node::Property(Box::new(node), list, operator);
                            }
                            Err(err) => { return Err(err); }
                        }
                    } else if symbols[2].compare(operator.value()) {
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
                return Ok(Node::Unary(Box::new(node.unwrap()), get_operator(operator)));
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
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), get_operator(operator));
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
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), get_operator(operator));
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
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), get_operator(operator));
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
                    node = Node::Binary(Box::new(node), Box::new(right.unwrap()), get_operator(operator));
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
                Err(Error::None) => {
                    return Err(err("parse_group", format!("expected expression"), 0));
                }
                Err(err) => { return Err(err); }
            }
            match self.skip_symbol(vec![end.clone(), vec![',' as u8]]) {
                Ok(tok) => {
                    if end.compare(tok.value()) {
                        return Ok(list);
                    }
                }
                Err(Error::None) => {
                    return Err(err("parse_group", format!("expected symbol \",\""), 0));
                }
                Err(err) => { return Err(err); }
            }
        }
    }
    fn parse_dict() {}
    fn parse_else(&mut self, key: Vec<u8>) -> Result<ast::Node> {
        //跳过边界
        match self.expect_type(TokenKind::RDelimiter) {
            Ok(_) => {}
            Err(err) => { return Err(err); }
        }
        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['{' as u8, '{' as u8, ], vec!['/' as u8], key]),
        ]));
        let mut body = vec![];
        match self.parse_until(&mut body) {
            Ok(_) => {}
            Err(Error::None) => {
                return Err(err("parse_else", format!("TODO:XX 命令未结束：语法/xx"), 0));
            }
            Err(err) => { return Err(err); }
        }
        self.pop_breakpoint();
        return Ok(Node::Else(body));
    }
    fn parse_if(&mut self, is_else_if: bool) -> Result<ast::Node> {
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
            Err(Error::None) => {
                return Err(err("parse_if", format!("TODO:if 命令未结束：必须至少包含 elif 或 else 或 /if其中之一"), 0));
            }
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
                    .compare(tok.value()) {
                    return self.parse_if(true);
                }
                //else
                if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ]
                    .compare(tok.value()) {
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
            Ok(_) => { return Ok(Node::If(Box::new(condition.unwrap()), body, items, is_else_if)); }
            Err(Error::None) => {
                return Err(err("parse_if", format!("TODO:if 命令未结束：必须以/if结束"), 0));
            }
            Err(err) => { return Err(err); }
        }
    }

    fn parse_for(&mut self) -> Result<ast::Node> {
        let mut key: Token;
        match self.expect_type(TokenKind::Identifier) {
            Ok(tok) => {
                key = tok;
            }
            Err(err) => {
                return Err(err);
            }
        }
        let mut value = Token::empty();
        match self.skip_symbol(vec![vec![',' as u8]]).and_then(|_| -> Result<Token>{
            self.expect_type(TokenKind::Identifier)
        }) {
            Ok(tok) => {
                value = tok;
            }
            Err(Error::None) => {}
            Err(err) => {
                return Err(err);
            }
        }

        match self.expect_value(vec![':' as u8]) {
            Ok(_) => {}
            Err(err) => {
                return Err(err);
            }
        }
        let mut expr: Node;
        match self.parse_expression() {
            Ok(node) => {
                expr = node;
            }
            Err(err) => {
                return Err(err);
            }
        }

        self.set_breakpoint(BreakPoint::build(vec![
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['{' as u8, '{' as u8, ], vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ]]),
            BreakPoint::new(true, TokenKind::Ignore, vec![vec!['{' as u8, '{' as u8, ], vec!['/' as u8], vec!['f' as u8, 'o' as u8, 'r' as u8, ]]),
        ]));
        let mut body = vec![];
        match self.parse_until(&mut body) {
            Ok(_) => {}
            Err(Error::None) => {
                return Err(err("parse_for", format!("TODO:for 命令未结束：必须至少包含 else 或 /for其中之一"), 0));
            }
            Err(err) => { return Err(err); }
        }
        self.pop_breakpoint();
        let mut for_else = Node::Empty;
        match self.skip_type(TokenKind::LDelimiter).and_then(|tok| -> Option<Token>{
            return self.skip_type(TokenKind::Identifier).or_else(|| -> Option<Token>{
                self.back(tok);
                return None;
            });
        }).ok_or(Error::None).and_then(|tok| -> Result<ast::Node> {
            //else
            if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ]
                .compare(tok.value()) {
                return self.parse_else(vec!['f' as u8, 'o' as u8, 'r' as u8, ]);
            }
            self.back(tok);
            return Err(Error::None);
        }) {
            Ok(node) => {
                for_else = node;
            }
            Err(Error::None) => {}
            err => { return err; }
        }

        match self.expect_type(TokenKind::LDelimiter)
            .and_then(|_| -> NoneResult{ self.expect_value(vec!['/' as u8]) })
            .and_then(|_| -> NoneResult{ self.expect_value(vec!['f' as u8, 'o' as u8, 'r' as u8, ]) }) {
            Ok(_) => { return Ok(Node::For(key, value, Box::new(expr), body, Box::new(for_else))); }
            Err(Error::None) => {
                return Err(err("parse_for", format!("TODO:for 命令未结束：必须以/for结束"), 0));
            }
            Err(err) => { return Err(err); }
        }
    }

    fn parse_print(&mut self, escape: bool) -> Result<ast::Node> {
        println!("parse_print");
        let mut body: Node;
        match self.parse_expression() {
            Ok(node) => {
                body = node;
            }
            err => { return err; }
        }
        return Ok(Node::Print(Box::new(body), escape));
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
                        if vec!['i' as u8, 'f' as u8, ].compare(tok.value()) {
                            return self.parse_if(false);
                        }
                        if vec!['f' as u8, 'o' as u8, 'r' as u8, ].compare(tok.value()) {
                            return self.parse_for();
                        }
                        self.back(tok);
                        return self.parse_print(true);
                    }
                    &TokenKind::Symbol => {
                        if vec!['!' as u8, '!' as u8, ].compare(tok.value()) {
                            return self.parse_print(false);
                        }
                        return Err(err("parse_statement", format!("unexpected symbol {}", tok.value_str()), tok.offset()));
                    }
                    _ => {
                        return Err(err("parse_statement", format!("unexpected token {}", tok.value_str()), tok.offset()));
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
        println!("parse");
        return self.take().and_then(|tok| -> Result<ast::Node>{
            match tok.kind() {
                &TokenKind::DomTagStart => {
                    return self.parse_dom_tag(tok).and_then(|mut node| -> Result<ast::Node>{
                        //                        match node {
                        //
                        //                            _ => {}
                        //                        }
                        return Ok(node);
                    });
                }
                &TokenKind::LDelimiter => {
                    return self.parse_statement();
                }
                &TokenKind::Data => {
                    //                    let (start, end) = optimize_literal(self.tokenizer.source().content(&tok));
                    //                    if end == 0 {
                    //                        return Ok(Node::Empty);
                    //                    }
                    //                    let tok = Token(TokenKind::Data, tok.1 + start, tok.1 + end);
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
        println!("parse_until");
        self.tokenizer.mark();
        loop {
            match self.check_breakpoint() {
                //println!("zzzzzzzzzzzzz");
                Ok(_) => {
                    self.tokenizer.unmark();
                    return self.extend_commands(buf);
                }
                Err(Error::EOF) => { break; }
                Err(Error::None) => {}
                err => { return err; }
            }

            match self.parse() {
                Ok(Node::Empty) => {}
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

    pub fn parse_all(&mut self) -> Result<NodeList> {
        println!("parse_all");
        let mut list = vec![];
        loop {
            match self.parse() {
                Ok(Node::Empty) => {}
                Ok(node) => { list.push(node) }
                Err(Error::None) | Err(Error::EOF) => { break; }
                Err(err) => { return Err(err); }
            }
        }
        match self.extend_commands(&mut list) {
            Ok(_) => {}
            Err(err) => { return Err(err); }
        };
        return Ok(list);
    }
    fn extend_if(&mut self, tag: Token, mut attrs: Vec<ast::DomAttr>, children: NodeList
                 , condition: Box<Node>
                 , others: &mut NodeList, is_else_if: bool) -> Result<Node> {
        println!("extend_if");
        let mut branches: NodeList = vec![];
        let mut size = others.len();
        while size > 0 && !others.is_empty() {
            size -= 1;
            let i = 0;
            let mut test = 0isize;
            match others[i] {
                Node::DomTag(_, ref next_attrs, _) => {
                    for next_attr in next_attrs {
                        if next_attr.name.2[0] != '@' as u8 {
                            continue;
                        }
                        let len = next_attr.name.2.len();
                        if vec!['e' as u8, 'l' as u8, 'i' as u8, 'f' as u8, ].compare(&next_attr.name.value()[1..len]) {
                            test = 1;
                        }
                        if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ].compare(&next_attr.name.value()[1..len]) {
                            test = 2;
                        }
                    }
                }
                _ => { test = 0; }
            }
            if test == 0 {
                continue;
            }
            let mut next = others.remove(i);
            if test == 1 {
                match next {
                    Node::DomTag(tag, mut attrs, children) => {
                        match self.extend_dom(tag, attrs, children, others, true) {
                            Ok(node) => { branches.push(node); }
                            Err(err) => { return Err(err); }
                        }
                    }
                    _ => {}
                }
            } else if test == 2 {
                match next {
                    Node::DomTag(tag, mut attrs, children) => {
                        // TODO: 扩展指令
                        while !attrs.is_empty() {
                            if attrs[0].name.2[0] != '@' as u8 {
                                continue;
                            }
                            let len = attrs[0].name.2.len();
                            if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ].compare(&attrs[0].name.value()[1..len]) {
                                attrs.remove(0);
                            }
                        }
                        branches.push(Node::Else(vec![Node::DomTag(tag, attrs, children)]));
                    }
                    _ => {}
                }
                break;
            }
        }
        let mut body = vec![Node::DomTag(tag, attrs, children)];
        match self.extend_commands(&mut body) {
            Ok(_) => {
                return Ok(Node::If(condition, body, branches, is_else_if));
            }
            Err(err) => { return Err(err); }
        }
    }

    fn extend_for(&mut self, tag: Token, mut attrs: Vec<ast::DomAttr>, children: NodeList
                  , key: Token, value: Token, iter: Box<Node>
                  , others: &mut NodeList) -> Result<Node> {
        println!("extend_for");
        let mut for_else = Node::Empty;
        let mut size = others.len();
        while size > 0 && !others.is_empty() {
            size -= 1;
            let i = 0;
            let mut test = 0isize;
            match others[i] {
                Node::DomTag(_, ref next_attrs, _) => {
                    for next_attr in next_attrs {
                        if next_attr.name.2[0] != '@' as u8 {
                            continue;
                        }
                        let len = next_attr.name.2.len();
                        if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ].compare(&next_attr.name.value()[1..len]) {
                            test = 2;
                        }
                    }
                }
                _ => { test = 0; }
            }
            if test == 0 {
                continue;
            }
            let mut next = others.remove(i);
            if test == 2 {
                match next {
                    Node::DomTag(tag, mut attrs, children) => {
                        // TODO: 扩展指令
                        while !attrs.is_empty() {
                            if attrs[0].name.2[0] != '@' as u8 {
                                continue;
                            }
                            let len = attrs[0].name.2.len();
                            if vec!['e' as u8, 'l' as u8, 's' as u8, 'e' as u8, ].compare(&attrs[0].name.value()[1..len]) {
                                attrs.remove(0);
                            }
                        }
                        for_else = Node::Else(vec![Node::DomTag(tag, attrs, children)]);
                        break;
                    }
                    _ => {}
                }
                break;
            }
        }
        let mut body = vec![Node::DomTag(tag, attrs, children)];
        match self.extend_commands(&mut body) {
            Ok(_) => {
                return Ok(Node::For(key, value, iter, body, Box::new(for_else)));
            }
            Err(err) => { return Err(err); }
        }
    }

    fn extend_dom(&mut self, tag: Token, mut attrs: Vec<ast::DomAttr>, children: NodeList
                  , list: &mut NodeList, is_else_if: bool) -> Result<Node> {
        println!("extend_dom");
        for i in 0..attrs.len() {
            if attrs[i].name.2[0] != '@' as u8 {
                continue;
            }
            let len = attrs[i].name.2.len();
            if len <= 1 {
                return Err(Error::Message("非法1".to_string()));
            }
            if vec!['i' as u8, 'f' as u8].compare(&attrs[i].name.value()[1..len])
                || vec!['e' as u8, 'l' as u8, 'i' as u8, 'f' as u8].compare(&attrs[i].name.value()[1..len]) {
                if is_else_if && vec!['i' as u8, 'f' as u8].compare(&attrs[i].name.value()[1..len]) {
                    return Err(Error::Message("错误的if扩展指令表达式".to_string()));
                }

                let mut attr = attrs.remove(i);
                if attr.value.len() == 0 {
                    println!("extend_dom:{:?}", attr);
                    return Err(Error::Message("非法".to_string()));
                }

                match attr.value.remove(0) {
                    Node::Statement(mut body) => {
                        if body.is_empty() {
                            return Err(Error::Message(format!("if/elif扩展指令必须包含表达式{:?}", attr)));
                        }
                        match body.remove(0) {
                            Node::If(condition, _, _, _) => {
                                return self.extend_if(tag, attrs, children, condition, list, is_else_if);
                            }
                            _ => { return Err(Error::Message("if扩展指令必须是条件表达式".to_string())); }
                        }
                    }
                    _ => {}
                }
                return Err(Error::Message("非法3".to_string()));
            } else if vec!['f' as u8, 'o' as u8, 'r' as u8].compare(&attrs[i].name.value()[1..len]) {
                let mut attr = attrs.remove(i);
                if attr.value.len() == 0 {
                    println!("extend_dom:{:?}", attr);
                    return Err(Error::Message("非法".to_string()));
                }

                match attr.value.remove(0) {
                    Node::Statement(mut body) => {
                        if body.is_empty() {
                            return Err(Error::Message(format!("if/elif扩展指令必须包含表达式{:?}", attr)));
                        }
                        match body.remove(0) {
                            Node::For(key, value, iter, _, _) => {
                                return self.extend_for(tag, attrs, children, key, value, iter, list);
                            }
                            _ => { return Err(Error::Message("if扩展指令必须是条件表达式".to_string())); }
                        }
                    }
                    _ => {}
                }
                return Err(Error::Message("非法3".to_string()));
            } else {
                return Err(Error::Message(format!("扩展指令不支持:{:?}", attrs[i].name.value_str())));
            }
        }
        return Ok(Node::DomTag(tag, attrs, children));
    }

    fn extend_commands(&mut self, list: &mut NodeList) -> NoneResult {
        println!("extend_commands");
        let mut buf: NodeList = vec![];
        while !list.is_empty() {
            let mut node = list.remove(0);
            match node {
                Node::DomTag(tag, mut attrs, children) => {
                    match self.extend_dom(tag, attrs, children, list, false) {
                        Ok(node) => { buf.push(node); }
                        Err(err) => { return Err(err); }
                    }
                }
                _ => { buf.push(node); }
            }
        }
        list.append(&mut buf);
        return Error::ok();
    }

    //    pub fn parse_root(&mut self) -> ast::Node {
    //        let mut root = ast::Root::new();
    //        self.parse_all(&mut root.body).expect("Failed to parse");
    //        return Node::Root(root);
    //    }
}
