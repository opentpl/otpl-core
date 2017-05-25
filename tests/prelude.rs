pub extern crate otpl;
//pub use self::otpl;
pub use self::otpl::parser;
pub use self::otpl::parser::Parser;
pub use self::otpl::scanner::{BytesScanner, Source};
pub use self::otpl::ast;
pub use self::otpl::ast::{Visitor, Node, NodeList};
pub use self::otpl::token;
pub use self::otpl::token::{Token,TokenKind};
use std::fs::OpenOptions;
use std::path::{Path};
use std::io;
use std::io::prelude::*;
use std::io::Write;
use self::otpl::util::VecSliceCompare;

pub fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
    return OpenOptions::new().read(true).open(path)
        .and_then(|mut f| -> io::Result<Vec<u8>>{
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).unwrap();
            return Ok(buf);
        }).expect("打开文件失败");
}


#[derive(Debug)]
pub struct StringWriter {
    output: Vec<u8>,
}

impl StringWriter {
    pub fn new() -> StringWriter {
        StringWriter { output: vec![] }
    }

    pub fn to_str(&mut self) -> String {
        String::from_utf8(self.output.clone()).unwrap()
    }
}

impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let len = buf.len();
        for c in buf {
            self.output.push(*c);
        }
        return Ok(len);
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}


pub struct Compiler<'a>(pub &'a Source, pub &'a mut Write, pub usize, pub bool);//indent
impl<'a> Compiler<'a> {
    fn gen_yield(&mut self) {
        if self.3 {
            self.1.write("yield ".as_ref());
        }
    }
    fn gen_indents(&mut self) {
        let mut s = String::new();
        for i in 0..self.2 * 4 {
            s += " ";
        }
        self.1.write(s.as_ref());
    }
    fn visit_list_format(&mut self, list: &NodeList) {
        for n in list {
            self.gen_indents();
            self.visit(&n);
            self.1.write("\n".as_ref());
        }
    }
}

impl<'a> Visitor for Compiler<'a> {
    fn visit_root(&mut self, body: &NodeList) {
        self.1.write("(function* () {\n".as_ref());
        self.2 += 1;
        self.3 = true;
        self.visit_list_format(body);
        self.3 = false;
        self.2 -= 1;
        self.1.write("})()".as_ref());
    }
    fn visit_dom_tag(&mut self, name: &Token, attrs: &Vec<ast::DomAttr>, children: &NodeList) {
        self.gen_yield();
        self.1.write("context.createElement('".as_ref());
        self.1.write(name.value());
        self.1.write("', {".as_ref());
        let mut first = false;
        for attr in attrs {
            if !first {
                first = true;
            } else {
                self.1.write(",".as_ref());
            }
            self.1.write("\"".as_ref());
            self.1.write(attr.name.value());
            self.1.write("\":".as_ref());
            self.visit_list(&attr.value);
        }
        self.1.write("}, (function* () {\n".as_ref());
        self.2 += 1;
        let prev_yield = self.3;
        self.3 = true;
        self.visit_list_format(children);
        self.3 = prev_yield;
        self.2 -= 1;
        self.gen_indents();
        self.1.write("})())".as_ref());
    }

    fn visit_literal(&mut self, tok: &Token) {
        self.gen_yield();
        self.1.write("`".as_ref());
        self.1.write(tok.value());
        self.1.write("`".as_ref());
    }
    fn visit_ternary(&mut self, expr: &Node, left: &Node, right: &Node) {
        self.1.write("visit_ternary".as_ref());
    }

    fn visit_binary(&mut self, left: &Node, right: &Node, operator: &Token) {
        self.visit(left);
        if vec!['+' as u8, ].compare(operator.value()) {
            self.1.write(" + ".as_ref());
        } else if vec!['-' as u8, ].compare(operator.value()) {
            self.1.write(" * ".as_ref());
        } else if vec!['*' as u8, ].compare(operator.value()) {
            self.1.write(" * ".as_ref());
        } else if vec!['/' as u8, ].compare(operator.value()) {
            self.1.write(" / ".as_ref());
        } else if vec!['%' as u8, ].compare(operator.value()) {
            self.1.write(" % ".as_ref());
        } else if vec!['<' as u8, ].compare(operator.value()) {
            self.1.write(" < ".as_ref());
        } else if vec!['>' as u8, ].compare(operator.value()) {
            self.1.write(" > ".as_ref());
        } else if vec!['=' as u8, '=' as u8, ].compare(operator.value()) {
            self.1.write(" == ".as_ref());
        } else if vec!['!' as u8, '=' as u8, ].compare(operator.value()) {
            self.1.write(" != ".as_ref());
        } else if vec!['<' as u8, '=' as u8, ].compare(operator.value()) {
            self.1.write(" <= ".as_ref());
        } else if vec!['>' as u8, '=' as u8, ].compare(operator.value()) {
            self.1.write(" >= ".as_ref());
        }
        self.visit(right);
    }

    fn visit_unary(&mut self, body: &Node, operator: &Token) {
        self.1.write("visit_unary".as_ref());
    }

    fn visit_property(&mut self, obj: &Node, params: &NodeList, operator: &Token) {
        self.1.write("visit_property".as_ref());
    }

    fn visit_method(&mut self, obj: &Node, params: &NodeList, operator: &Token) {
        self.1.write("visit_method".as_ref());
    }

    fn visit_string(&mut self, tok: &Token) {
        self.1.write("visit_string".as_ref());
    }

    fn visit_boolean(&mut self, tok: &Token) {
        self.1.write("visit_boolean".as_ref());
    }

    fn visit_integer(&mut self, tok: &Token) {
        self.1.write(tok.value());
    }

    fn visit_float(&mut self, integer: &Token, decimal: &Token) {
        self.1.write("visit_float".as_ref());
    }

    fn visit_none(&mut self, tok: &Token) {
        unimplemented!()
    }

    fn visit_identifier(&mut self, tok: &Token) {
        self.1.write("context.get('".as_ref());
        self.1.write(tok.value());
        self.1.write("')".as_ref());
    }

    fn visit_if(&mut self, condition: &Node, body: &NodeList, branches: &NodeList,is_else_if:&bool) {
        if *is_else_if{
            self.1.write("else if".as_ref());
        }else {
            self.1.write("if".as_ref());
        }
        self.1.write(" (".as_ref());
        self.visit(condition);
        self.1.write(") {\n".as_ref());
        self.2 += 1;
        self.visit_list_format(body);
        self.2 -= 1;
        self.gen_indents();
        self.1.write("}".as_ref());
        self.visit_list(branches);
    }

    fn visit_else(&mut self, body: &NodeList) {
        self.1.write("else {\n".as_ref());
        self.2 += 1;
        self.visit_list_format(body);
        self.2 -= 1;
        self.gen_indents();
        self.1.write("}".as_ref());
    }
    fn visit_for(&mut self, key: &Token, value: &Token, iter: &Node, body: &NodeList, for_else: &Node) {
        //TODO: 临时随机变量
        self.1.write("for (let ".as_ref());
        if let &TokenKind::Ignore=value.kind(){
            self.1.write(key.value());
            self.1.write(" of context.toArray(".as_ref());
            self.visit(iter);
        }else {
            self.1.write("[".as_ref());
            self.1.write(key.value());
            self.1.write(", ".as_ref());
            self.1.write(value.value());
            self.1.write("] of context.toMap(".as_ref());
            self.visit(iter);
        }
        self.1.write(") {\n".as_ref());
        self.2 += 1;
        self.visit_list_format(body);
        self.2 -= 1;
        self.gen_indents();
        self.1.write("}".as_ref());
        //TODO: else
    }
}