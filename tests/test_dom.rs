mod prelude;
use self::prelude::*;

#[test]
#[ignore]
fn test_pure_dom() {
    let buf = read_file("./tests/dom_pure.html");

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