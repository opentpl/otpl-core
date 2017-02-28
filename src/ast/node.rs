
//https://www.oschina.net/question/81620_239264

/// 用于记录节点位于原代码中的位置，展开(lineNo, column)。
#[derive(Debug)]
pub struct Pos(pub i32, pub i32);

/// 定义的一个语法树的节点集合。
pub type NodeList<'a> = Vec<Node<'a>>;

/// 定义的一个语法树的节点。
#[derive(Debug)]
pub enum Node<'a> {
    /// 表是一个用于占位的空节点。
    None,
    /// 表是一个 DOM 节点，如：div。
    DomNode {
        pos: Pos,
        tag: &'a str,
        attrs: NodeList<'a>,
        children: NodeList<'a>,
    },
    /// 表示一个 DOM 节点的属性，如： id。
    DomAttr {
        pos: Pos,
        name: &'a str,
        value: Box<Node<'a>>,
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;
    #[test]
    fn test() {
        let attr = Node::DomAttr {
            pos: Pos(0, 0),
            name: "div",
            value: Box::new(Node::None)
        };
        let node = Node::DomNode {
            pos: Pos(0, 0),
            tag: "div",
            attrs: vec![attr],
            children: vec![],
        };

        println!("{:?}", node);
    }
}

