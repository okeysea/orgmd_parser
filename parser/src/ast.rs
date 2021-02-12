use serde::Serialize;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;

type Link = Rc<RefCell<ASTNode>>;

#[derive(Debug, PartialEq, Serialize)]
pub struct ASTNode {
    #[serde(flatten)]
    data: ASTElm,

    children: Vec<Link>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct ASTElm {
    pub elm_type: ASTType,
    pub elm_meta: ASTMetaData,
    pub value: String,
    pub raw_value: String, // パース前のデータ
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ASTType {
    Document,
    Paragraph,
    Headers,
    Text,
    Emphasis,
    SoftBreak,
}

impl Default for ASTType {
    fn default() -> Self {
        ASTType::Document
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ASTMetaData {
    Nil,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Default for ASTMetaData {
    fn default() -> Self {
        ASTMetaData::Nil
    }
}

impl ASTNode {
    pub fn new(v: ASTElm) -> Self {
        ASTNode {
            data: v,
            children: vec![],
        }
    }

    pub fn append(&mut self, v: ASTElm) {
        let node = ASTNode::new(v);
        self.append_node(node);
    }

    pub fn append_node(&mut self, node: ASTNode) {
        let rc_node = Rc::new(RefCell::new(node));
        self.children.push(Rc::clone(&rc_node));
    }

    pub fn append_node_from_vec(&mut self, nodes: Vec<ASTNode>) {
        for node in nodes {
            self.append_node(node);
        }
    }

    //
    // --- setter, getter ---
    //

    pub fn node_type(&self) -> &ASTType {
        &self.data.elm_type
    }

    pub fn meta(&self) -> &ASTMetaData {
        &self.data.elm_meta
    }

    pub fn value(&self) -> &String {
        &self.data.value
    }

    pub fn raw_value(&self) -> &String {
        &self.data.raw_value
    }

    pub fn node_type_mut(&mut self) -> &mut ASTType {
        &mut self.data.elm_type
    }

    pub fn meta_mut(&mut self) -> &mut ASTMetaData {
        &mut self.data.elm_meta
    }

    pub fn value_mut(&mut self) -> &mut String {
        &mut self.data.value
    }

    pub fn raw_value_mut(&mut self) -> &mut String {
        &mut self.data.raw_value
    }

    pub fn set_node_type(&mut self, v: ASTType) {
        *self.node_type_mut() = v;
    }

    pub fn set_meta(&mut self, v: ASTMetaData) {
        *self.meta_mut() = v;
    }

    pub fn set_value(&mut self, v: String) {
        *self.value_mut() = v;
    }

    pub fn set_raw_value(&mut self, v: String) {
        *self.raw_value_mut() = v;
    }

    //
    // --- rendering ---
    //
    pub fn render_debug_format(&self) -> String {
        self._render_debug_format(self)
    }

    fn _render_tag(&self, tagname: &str, node: &ASTNode) -> String {
        let mut result: String = "<".to_string() + tagname + ">";
        for child in &node.children {
            result = result + &self._render_debug_format(&child.borrow());
        }
        result = result + "</" + tagname + ">";
        return result;
    }

    fn _render_debug_format(&self, node: &ASTNode) -> String {
        let mut result: String = "".to_string();
        match node.node_type() {
            ASTType::Document => {
                result += &self._render_tag("document", &node);
            }
            ASTType::Paragraph => {
                result += &self._render_tag("paragraph", &node);
            }
            ASTType::Headers => {
                result += &self._render_tag("header", &node);
            }
            ASTType::Text => {
                result += &("<text>".to_string() + &node.value().to_string() + "</text>");
            }
            ASTType::Emphasis => {
                result += &self._render_tag("emphasis", &node);
            }
            ASTType::SoftBreak => {
                //result = node.value().to_string();
                result += "<softbreak />";
            }
            _ => {
                result = result + "<error>Undefined ASTNode Type</error>";
            }
        }
        return result;
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

/*
#[cfg(test)]
mod tests {
    use super::*;

    fn ASTNode_return() -> ASTNode {
        ASTNode::new( ASTElm {
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "astnode_return".to_string(),
        })
    }

    fn ASTNode_markdown_header() -> ASTNode {

        let mut node = ASTNode::new( ASTElm {
            elm_type: ASTType::Headers,
            elm_meta: ASTMetaData::H1,
            value: "".to_string(),
        });

        node.append( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "Headering 1".to_string(),
        });

        return node
    }

    fn ASTNode_markdown_paragraph() -> ASTNode {

        let mut node = ASTNode::new( ASTElm {
            elm_type: ASTType::Paragraph,
            elm_meta: ASTMetaData::Nil,
            value: "".to_string(),
        });

        node.append( ASTElm{
            elm_type: ASTType::Text,
            elm_meta: ASTMetaData::Nil,
            value: "This is Paragraph block.".to_string(),
        });

        return node
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

        println!("ASTNode_basics tests");
        println!("{:?}", node);
        println!("{:?}", node.node_type());
    }

    #[test]
    fn ASTNode_markdown() {
        let mut node_empty = ASTNode::new(Default::default());
        let mut node = ASTNode::new( ASTElm{
            elm_type: ASTType::Document,
            elm_meta: ASTMetaData::Nil,
            value: "# Headering 1\nThis is paragraph block.".to_string(),
        } );

        node.append_node( ASTNode_markdown_header() );
        node.append_node( ASTNode_markdown_paragraph() );

        println!("ASTNode_markdown tests");
        println!("{:?}", node);
        println!("{:?}", node_empty);
    }
}*/
