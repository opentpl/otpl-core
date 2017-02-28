use std::any::Any;

/// 节点类型。
#[derive(Debug)]
pub enum NodeType{
    DomNode = 1,
    DomAttr =2,
}

/// 定义的一个语法树的节点。
#[derive(Debug)]
pub enum  Node{
    DomNode(i32, i32, Option<Box<Any>>),
    DomAttr(i32, i32, Box<Any>),
}

//impl Node {
//    fn new(kind: NodeType, line: i32, column: i32, payload: Option) -> Node{
//        Node{
//            kind: kind,
//            payload: None,
//        }
//    }
//}

//struct DomNode{
//    attrs: Vec<Box<i32>>,
//    children: Vec<Node>,
//}
struct DomAttr<'a>{
    name: &'a str,
    value: Node,
}

#[test]
fn test(){
    let inner = Node::DomNode(0,0,None);
    let node = Node::DomNode(0,10,Some(Box::new(DomAttr{name:"", value: inner})));

    println!("{:?}", node);

    let p = match node{
        Node::DomNode(_,_,payload) => payload,
        _=>None,
    };
    println!("{:?}", p);

//    let mut dnode= DomNode{attrs:vec![],children:vec![]};
//
//    let node = Node::new(NodeType::DomAttr, 0, 1, DomAttr{name: "", value: None});
//    dnode.attrs.push(node);

}