use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;

type Link<T> = Rc<RefCell<ASTNode<T>>>;

#[derive(Debug)]
pub struct ASTNode<T> {
    data: T,
    children: Vec<Link<T>>,
}

#[derive(Debug)]
pub struct ASTElm {
    elm_type: ASTType,
    elm_meta: ASTMetaData,
    value: String,
}

#[derive(Debug)]
pub enum ASTType {
    Document,
    Paragraph,
    Headers,
    Text,
}

#[derive(Debug)]
pub enum ASTMetaData {
    Nil,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl<T> ASTNode<T> {
    pub fn new(v: T) -> Self{
        ASTNode {
            data: v,
            children: vec![],
        }
    }

    pub fn append(&mut self, v: T){
        let node = ASTNode::new(v);
        let rc_node = Rc::new(RefCell::new(node));
        self.children.push( Rc::clone( &rc_node ) );
    }

    pub fn append_node(&mut self, node: ASTNode<T>){
        let rc_node = Rc::new(RefCell::new(node));
        self.children.push( Rc::clone( &rc_node ) );
    }

    pub fn type(&self) -> &ASTType {
        &self.data.elm_type
    }

    pub fn meta(&self) -> &ASTMetaData {
        &self.data.elm_meta
    }

    pub fn value(&self) -> &String {
        &self.data.value
    }
}

// イテレータの実装
/*
impl<T> Iterator for ASTNode<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {

    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    fn ASTNode_return() -> ASTNode<ASTElm> {
        ASTNode::new( ASTElm {
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "astnode_return".to_string(),
        })
    }

    #[test]
    fn ASTNode_basics() {
        let mut node = ASTNode::new( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "hello".to_string(),
        } );

        node.append( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "Hello2".to_string(),
        });

        node.append_node( ASTNode::new( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "hello3".to_string(),
        } ) );

        node.append_node( ASTNode_return() );

        println!("{:?}", node);
    }
}
