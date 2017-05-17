mod prelude;
use self::prelude::*;

use std::io::Write;

#[derive(Debug)]
struct StringWriter {
    output:Vec<u8>,
}
impl StringWriter {
    fn new() -> StringWriter{
        StringWriter{ output:vec![]}
    }

    fn to_str(&mut self)->String{
            String::from_utf8(self.output.clone()).unwrap()
    }
}

impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize,std::io::Error>{
        let len = buf.len();
        for c in buf {
            self.output.push(*c);
        }
        return Ok(len);
    }
    fn flush(&mut self) -> Result<(),std::io::Error>{
        Ok(())
    }
}


struct Compiler<'a>(&'a Source,&'a mut Write,usize);//indent
impl<'a> Compiler<'a>{
    fn optimize_literal(value:&[u8])->&[u8]{
        let mut start =0usize;
        let mut end =value.len();
        for i in 0..value.len() {
            let ch=value[i];
            if !(ch== ('\r' as u8) || ch== ('\n' as u8) || ch== ('\t' as u8) ||ch== (' ' as u8)){
                start=i;
                break;
            }
        }
        for i in (0..value.len()).rev() {
            let ch=value[i];
            if !(ch== ('\r' as u8) || ch== ('\n' as u8) || ch== ('\t' as u8) ||ch== (' ' as u8)){
                end=i+1;
                break;
            }
            end=i;
        }
        println!("abc:{} {} {}", start, end, value.len());
        return &value[start..end];
    }

    fn gen_indents(&mut self){
        let mut s=String::new();
        for i in 0..self.2*4 {
            s+=".";
        }
        self.1.write(s.as_ref());
    }
}
impl<'a> Visitor for Compiler<'a> {
    fn visit_dom_tag(&mut self, tag: &ast::DomTag) {
        self.gen_indents();
        self.1.write("xview.createElement(xview.getDenined('".as_ref());
        self.1.write(self.0.content(&tag.name));
        self.1.write("'),xview.procProperties({".as_ref());
        let mut first = false;
        for attr in &tag.attrs {
            if !first {
                first = true;
            }else {
                self.1.write(",".as_ref());
            }
            self.1.write("\"".as_ref());
            self.1.write(self.0.content(&attr.name));
            self.1.write("\":".as_ref());
            self.visit_list(&attr.value);
        }
        self.1.write("}),[\n".as_ref());
        self.2+=1;
        for n in &tag.children {
            match n{
                &Node::Literal(ref tok) => {
                    let content=Compiler::optimize_literal(self.0.content(&tok));
                    if content.len()==0{
                        continue;
                    }
                    self.gen_indents();
                    self.1.write("`".as_ref());
                    self.1.write(content);
                    self.1.write("`".as_ref());
                }
                _=>{self.visit(&n);}
            }
            self.1.write(",\n".as_ref());
        }
        self.2-=1;
        self.gen_indents();
        self.1.write("])".as_ref());

    }

    fn visit_literal(&mut self, tok: &Token) {
        self.gen_indents();
        self.1.write("`".as_ref());
        self.1.write(self.0.content(&tok));
        self.1.write("`".as_ref());
    }
    fn visit_ternary(&mut self, node: &Node, left: &Node, right: &Node) {
        unimplemented!()
    }
}

#[test]
//#[ignore]
fn test_pure_dom() {
    let buf = read_file("./tests/pure_dom.html");

    let mut scanner = BytesScanner::new(&buf, "source".as_ref());
    let root: ast::Node;
    {
        let mut parser = Parser::new(&mut scanner);
        root = parser.parse_root();
        println!("Parse Done! ==============================");
    }

    {
        let mut writer=StringWriter::new();
        {
            let mut visitor = Compiler(&scanner,&mut writer,0);
            visitor.visit(&root);
            println!("Visit Done! ==============================");
        }

        println!("{}",writer.to_str());

    }
    //end
}