extern crate otpl;

use otpl::parser::Parser;
use otpl::scanner::{BytesScanner, Source};
use otpl::ast;
use otpl::ast::Visitor;
use otpl::token::Token;
use std::fs::OpenOptions;
use std::path::{Path};
use std::io::prelude::*;


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
        println!("literal=> {:?}", self.0.content_str(tok));
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
    return OpenOptions::new().read(true).open(path)
        .and_then(|mut f| -> std::io::Result<Vec<u8>>{
            let mut buf = Vec::new();
            f.read_to_end(&mut buf);
            return Ok(buf);
        }).expect("打开文件失败");
}

#[test]
fn test_dom() {
    let buf = read_file("./src/scanner/test.html");
    //

    let mut scanner = BytesScanner::new(&buf, "source".as_ref());
    let root: ast::Node;
    {
        let mut parser = Parser::new(&mut scanner);
        root = parser.parse_root();
        println!("Parse Done! ==============================");
    }

    {
        let mut visitor = TestVisitor(&scanner);
        visitor.visit(&root);
        println!("Visit Done! ==============================");
    }
    //end
}