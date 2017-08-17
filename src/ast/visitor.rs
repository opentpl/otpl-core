use super::{Node, NodeList, DomAttr,Operator,Constant};
use token::Token;
use {Error, Result};

pub type VisitResult = Result<()>;

/// 定义一个用于访问AST节点的一组方法。
pub trait Visitor {
    /// 访问分类抽象节点。
    fn visit(&mut self, node: &Node) -> VisitResult {
        return match node {
            &Node::Root(ref inner) => self.visit_root(inner),
            &Node::Literal(ref inner) => self.visit_literal(inner),
            &Node::DomTag(ref name, ref attrs, ref children) => self.visit_dom_tag(name, attrs, children),
            &Node::Statement(ref inner) => self.visit_statement(inner),
            &Node::Ternary(ref expr, ref left, ref right) => self.visit_ternary(expr, left, right),
            &Node::Binary(ref left, ref right, ref operator) => self.visit_binary(left, right, operator),
            &Node::Unary(ref body, ref operator) => self.visit_unary(body, operator),
            &Node::Property(ref obj, ref params, ref operator) => self.visit_property(obj, params, operator),
            &Node::Method(ref obj, ref params, ref operator) => self.visit_method(obj, params, operator),
            &Node::Const(ref inner) => self.visit_const(inner),
            &Node::Identifier(ref inner) => self.visit_identifier(inner),
            &Node::If(ref condition, ref body, ref branches, ref is_else_if) => self.visit_if(condition, body, branches, is_else_if),
            &Node::Else(ref body) => self.visit_else(body),
            &Node::For(ref key, ref val, ref iter, ref body, ref for_else) => self.visit_for(key, val, iter, body, for_else),
            &Node::Print(ref body, ref escape) => self.visit_print(body, escape),
            &Node::Array(ref inner) => self.visit_array(inner),
            &Node::Map(ref inner) => self.visit_map(inner),
            &Node::MapEntry(ref key,ref val) => self.visit_map_entry(key,val),
            _ => self.visit_undefined(node)
        }
    }
    /// 访问未在本访问器定义的 Node。
    #[allow(unused_variables)]
    fn visit_undefined(&mut self, node: &Node) -> VisitResult {
        match node {
            &Node::Empty => {}
            _ => println!("warning: undefined visit node {:?}", node)
        }
        return Ok(());
    }
    fn visit_root(&mut self, body: &NodeList) -> VisitResult {
        self.visit_list(body)
    }
    fn visit_list(&mut self, list: &NodeList) -> VisitResult {
        for n in list {
            match self.visit(&n) {
                Ok(_) | Err(Error::None) => {}
                err @ _ => { return err; }
            };
        }
        return Ok(());
    }
    /// 访问字面量
    fn visit_literal(&mut self, tok: &Token) -> VisitResult;
    /// 访问 DomTag
    fn visit_dom_tag(&mut self, name: &Token, attrs: &Vec<DomAttr>, children: &NodeList) -> VisitResult;
    fn visit_statement(&mut self, body: &NodeList) -> VisitResult {
        self.visit_list(body)
    }
    fn visit_ternary(&mut self, expr: &Node, left: &Node, right: &Node) -> VisitResult;
    fn visit_binary(&mut self, left: &Node, right: &Node, operator: &Operator) -> VisitResult;
    fn visit_unary(&mut self, body: &Node, operator: &Operator) -> VisitResult;
    fn visit_property(&mut self, obj: &Node, params: &NodeList, operator: &Token) -> VisitResult;
    fn visit_method(&mut self, obj: &Node, params: &NodeList, operator: &Token) -> VisitResult;
    fn visit_const(&mut self, tok: &Constant) -> VisitResult;
    fn visit_identifier(&mut self, tok: &Token) -> VisitResult;
    fn visit_if(&mut self, condition: &Node, body: &NodeList, branches: &NodeList, is_else_if: &bool) -> VisitResult;
    fn visit_else(&mut self, body: &NodeList) -> VisitResult {
        self.visit_list(body)
    }
    fn visit_for(&mut self, key: &Token, value: &Token, iter: &Node, body: &NodeList, for_else: &Node) -> VisitResult;
    fn visit_print(&mut self, body: &Node, escape: &bool) -> VisitResult;
    fn visit_array(&mut self, items: &NodeList) -> VisitResult;
    fn visit_map(&mut self, entries: &NodeList) -> VisitResult;
    fn visit_map_entry(&mut self, key: &Token, value: &Node) -> VisitResult;
}