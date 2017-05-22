mod prelude;

use self::prelude::*;

use std::io::Write;
use self::prelude::otpl::util::VecSliceCompare;

#[derive(Debug)]
struct StringWriter {
    output: Vec<u8>,
}

impl StringWriter {
    fn new() -> StringWriter {
        StringWriter { output: vec![] }
    }

    fn to_str(&mut self) -> String {
        String::from_utf8(self.output.clone()).unwrap()
    }
}

impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let len = buf.len();
        for c in buf {
            self.output.push(*c);
        }
        return Ok(len);
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}


struct Compiler<'a>(&'a Source, &'a mut Write, usize,bool);//indent
impl<'a> Compiler<'a> {
    fn gen_yield(&mut self){
        if self.3{
            self.1.write("yield ".as_ref());
        }
    }
    fn gen_indents(&mut self) {
        let mut s = String::new();
        for i in 0..self.2 * 4 {
            s += ".";
        }
        self.1.write(s.as_ref());
    }
}

impl<'a> Visitor for Compiler<'a> {
    fn visit_root(&mut self, body: &NodeList) {
        self.1.write("xview.GeneratorToArray((function* () {\n".as_ref());
        self.2 += 1;
        self.3=true;
        for n in body {
            self.gen_indents();
            self.visit(&n);
            self.1.write("\n".as_ref());
        }
        self.3=false;
        self.2 -= 1;
        self.1.write("})())".as_ref());
    }
    fn visit_dom_tag(&mut self, name: &Token, attrs: &Vec<ast::DomAttr>, children: &NodeList) {
        self.gen_yield();
        self.1.write("xview.createElement(xview.getDenined('".as_ref());
        self.1.write(name.value());
        self.1.write("'),xview.procProperties({".as_ref());
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
        self.1.write("}),xview.GeneratorToArray((function* () {\n".as_ref());
        self.2 += 1;
        let prev_yield=self.3;
        self.3=true;
        for n in children {
            self.gen_indents();
            self.visit(&n);
            self.1.write("\n".as_ref());
        }
        self.3=prev_yield;
        self.2 -= 1;
        self.gen_indents();
        self.1.write("})()))".as_ref());
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
        if vec!['+' as u8,].compare(operator.value()){
            self.1.write(" + ".as_ref());
        } else if vec!['-' as u8,].compare(operator.value()){
            self.1.write(" * ".as_ref());
        } else if vec!['*' as u8,].compare(operator.value()){
            self.1.write(" * ".as_ref());
        } else if vec!['/' as u8,].compare(operator.value()){
            self.1.write(" / ".as_ref());
        } else if vec!['%' as u8,].compare(operator.value()){
            self.1.write(" % ".as_ref());
        } else if vec!['<' as u8,].compare(operator.value()){
            self.1.write(" < ".as_ref());
        } else if vec!['>' as u8,].compare(operator.value()){
            self.1.write(" > ".as_ref());
        } else if vec!['=' as u8,'=' as u8,].compare(operator.value()){
            self.1.write(" == ".as_ref());
        } else if vec!['!' as u8,'=' as u8,].compare(operator.value()){
            self.1.write(" != ".as_ref());
        } else if vec!['<' as u8,'=' as u8,].compare(operator.value()){
            self.1.write(" <= ".as_ref());
        } else if vec!['>' as u8,'=' as u8,].compare(operator.value()){
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

    fn visit_if(&mut self, condition: &Node, body: &NodeList, branches: &NodeList) {
        self.1.write("if (".as_ref());
        self.visit(condition);
        self.1.write(") {\n".as_ref());
        self.2 += 1;
        self.visit_list(body);
        self.2 -= 1;
        self.1.write("}".as_ref());
        self.visit_list(branches);
    }

    fn visit_else(&mut self, body: &NodeList) {
        unimplemented!()
    }
}

#[test]
#[ignore]
fn test_pure_dom() {
    let buf = read_file("./tests/pure_dom.html");

    let mut scanner = BytesScanner::new(&buf, "source".as_ref());
    let root: self::prelude::otpl::Result<ast::NodeList>;
    {
        let mut parser = Parser::new(&mut scanner);
        root = parser.parse_all();
        println!("Parse Done! ==============================");
    }

    {
        let mut writer = StringWriter::new();
        {
            let mut visitor = Compiler(&scanner, &mut writer, 0,false);
            let root = Node::Root(root.expect("Failed to parse"));
            visitor.visit(&root);
            println!("Visit Done! ==============================");
        }

        println!("{}", writer.to_str());
    }
    //end
}

#[test]
//#[ignore]
fn test_extend_if() {
    let buf = read_file("./tests/dom_extend_if.html");

    let mut scanner = BytesScanner::new(&buf, "source".as_ref());
    let root: ast::NodeList;
    {
        let mut parser = Parser::new(&mut scanner);
        root = parser.parse_all().expect("Failed to parse");
        println!("Parse Done! ==============================");
    }

    {
        let mut writer = StringWriter::new();
        {
            let mut visitor = Compiler(&scanner, &mut writer, 0,false);
            let root = Node::Root(root);
            visitor.visit(&root);
            println!("Visit Done! ==============================");
        }

        println!("{}", writer.to_str());
    }
    //end
}