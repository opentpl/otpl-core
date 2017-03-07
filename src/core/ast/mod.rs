mod node;
mod dom;
mod visitor;

pub use self::node::*;
pub use self::dom::*;
pub use self::visitor::*;

#[cfg(test)]
mod tests {
    use super::*;

    struct TestVisitor;

    impl Visitor for TestVisitor {
        fn visit_dom_node(&mut self, node: &DomNode) {
            println!(">visit_dom_node: {:?}", node);
            for attr in &node.attrs {
                self.visit(&attr.value)
            }
        }
    }

    #[test]
    fn define_and_visit() {
        let mut dnode = DomNode::new(10, 0, "div");
        dnode.attrs.push(DomAttr::new(0, 50, "id", Node::None));
        let node = Node::DomNode(dnode);
        println!("{:?}", node);
        let mut visitor = TestVisitor;
        visitor.visit(&node);
    }

    trait Foo{
        fn to(&self) ->i32;
    }

    fn test_fn(f: Box<Foo>){
    }

}