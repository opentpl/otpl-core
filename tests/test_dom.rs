mod prelude;

use self::prelude::*;

fn compile(source: &[u8]) {
    let mut scanner = BytesScanner::new(source, "source".as_ref());
    let root: ast::NodeList;
    {
        let mut parser = Parser::new(&mut scanner);
        root = parser.parse_all().expect("Parse Error");
        println!("Parse Done! ==============================");
    }

    //    {
    //        let mut writer = StringWriter::new();
    //        {
    //            let mut visitor = Compiler(&scanner, &mut writer, 0,false);
    //            let root = Node::Root(root.expect("Failed to parse"));
    //            visitor.visit(&root);
    //            println!("Visit Done! ==============================");
    //        }

    //println!("{}", writer.to_str());
    //}
    //end
}


#[test]
//#[ignore]
fn test_pure_dom() {
    let buf = "<div class=\"wrap\">
    <h1>This is a pure dom page</h1>
    <hr>
    <Custom.Sub :value=\"val\"></Custom.Sub>
    line 1<br/>
    line2
    <p style=\"color: gray\">line end</p>
</div>aa";
    compile(buf.as_ref());
}

#[test]
//#[ignore]
fn test_extend_if() {
    let buf = "<div @if=\"i==0\">if</div>
<div @elif=\"i==1\">else if</div>
<div @else>
    else
    <div @if=\"i==0\">nesting if</div>
    <div @elif=\"i==1\">nesting else if</div>
    <div @else> nesting else</div>
</div>";
    compile(buf.as_ref());
}

#[test]
//#[ignore]
fn test_extend_for() {
    let buf = "<div @for=\"i : arr\">{{i}}asd</div>";
    compile(buf.as_ref());
}

#[test]
//#[ignore]
fn test_binds() {
    let buf = "<div class={{['button']}}>{{i}}asd</div>";
    compile(buf.as_ref());
}